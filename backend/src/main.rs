use std::sync::Arc;

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::Method,
    response::IntoResponse,
    routing::get,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{debug, error, info};
use tokio::{
    net::TcpListener,
    sync::{
        Mutex,
        broadcast::{self, Receiver, Sender},
    },
};
use tower::ServiceBuilder;
use tower_cookies::{CookieManagerLayer, Cookies};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use math_battle::{User, UserId};

use crate::{
    dto::Request,
    state::{AppState, OwnerId},
};

mod client_cookies;
mod dto;
mod request_handler;
mod state;

#[derive(Debug, Default)]
pub struct Session {
    user_id: UserId,
    room_id: Option<OwnerId>,
}

impl Session {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventBus<E, const N: usize = 100>
where
    E: Clone,
{
    pub tx: Sender<E>,
}

impl<E, const N: usize> Default for EventBus<E, N>
where
    E: Clone,
{
    fn default() -> Self {
        let (tx, _) = broadcast::channel::<E>(N);

        Self { tx }
    }
}

impl<E, const N: usize> EventBus<E, N>
where
    E: Clone,
{
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    Data {
        response: dto::Response,
        share_with: EventShare,
    },
    Close(UserId),
}

impl Event {
    pub fn new(response: dto::Response, share_with: EventShare) -> Self {
        Self::Data {
            response,
            share_with,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventShare {
    One(UserId),
    Many(Vec<UserId>),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().expect("failed to initialize .env");
    tracing_subscriber::fmt::init();

    let addr = dotenvy::var("ADDR").expect("environment variable `ADDR` must be set");

    let listener = TcpListener::bind(&addr).await?;
    let router = Router::new()
        .route("/api/v1/ws", get(ws))
        .layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(CorsLayer::new().allow_methods([Method::GET])),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(AppState::default());

    info!("Listening on: {addr}...");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn ws(
    State(state): State<AppState>,
    cookies: Cookies,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    info!("Got request for handshake");

    let session = Arc::new(Mutex::new(Session::new(client_cookies::set_user_id(
        cookies,
    ))));

    ws.on_upgrade(move |socket| handle_socket(socket, state, session))
}

async fn handle_socket(socket: WebSocket, mut state: AppState, session: Arc<Mutex<Session>>) {
    let (sender, receiver) = socket.split();

    // Add user session
    state
        .add_user(User::from(session.lock().await.user_id))
        .await;

    let tx = state.tx();

    let _ = tokio::join! {
        tokio::spawn(reader(receiver, tx.clone(), state.clone(), Arc::clone(&session))),
        tokio::spawn(writer(sender, tx.subscribe(), Arc::clone(&session)))
    };

    // After disconnected
    if session.lock().await.room_id.is_some() {
        // Remove user from quiz
        let _ = request_handler::quiz_leave(tx.clone(), &mut state, Arc::clone(&session)).await;

        // Remove user from room
        let _ = request_handler::room_leave(tx.clone(), &mut state, Arc::clone(&session)).await;
    }

    // Drop user session
    state.remove_user(&session.lock().await.user_id).await;
}

async fn reader(
    mut receiver: SplitStream<WebSocket>,
    tx: Sender<Event>,
    mut state: AppState,
    session: Arc<Mutex<Session>>,
) {
    while let Some(Ok(msg)) = receiver.next().await {
        let user_id = { session.lock().await.user_id };

        if let Message::Close(_) = msg {
            info!("Got request to close connection from {:?}", user_id);
            let _ = tx.send(Event::Close(user_id));
            break;
        }

        let bytes = match msg.into_text() {
            Ok(data) => data,
            Err(e) => {
                error!("failed to read message: {e}");
                let _ = tx.send(Event::Close(user_id));
                break;
            }
        };

        let req = match serde_json::from_str::<Request>(bytes.as_str()) {
            Ok(data) => data,
            Err(e) => {
                error!("failed to parse message: {e}");
                continue;
            }
        };

        let tx = tx.clone();
        let state = &mut state;
        let session = Arc::clone(&session);

        let _ = match req {
            // User requests
            Request::User => request_handler::user(tx, state, session).await,
            Request::UsernameUpdated { username } => {
                request_handler::username_updated(tx, state, session, username).await
            }

            // Room requests
            Request::RoomCreate => request_handler::room_create(tx, state, session).await,
            Request::RoomJoin { room_id } => {
                request_handler::room_join(tx, state, session, room_id).await
            }
            Request::RoomLeave => request_handler::room_leave(tx, state, session).await,
            Request::RoomMembersList => {
                request_handler::room_members_list(tx, state, session).await
            }

            // Quiz requests
            Request::QuizStart => request_handler::start_quiz(tx, state, session).await,
            Request::QuizJoin { room_id } => {
                unimplemented!()
            }
            Request::QuizLeave => request_handler::quiz_leave(tx, state, session).await,
            Request::QuizQuestion => request_handler::quiz_question(tx, state, session).await,
            Request::QuizCheck {
                question_id,
                variant_id,
            } => request_handler::quiz_check(tx, state, session, question_id, variant_id).await,
            Request::QuizLeaderboard => request_handler::quiz_leaderboard(tx, state, session).await,
            Request::QuizTimer => request_handler::quiz_timer(tx, state, session).await,
        };
    }

    debug!(
        "Reader was closed for user {:?}",
        session.lock().await.user_id
    );
}

async fn writer(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: Receiver<Event>,
    session: Arc<Mutex<Session>>,
) {
    while let Ok(event) = rx.recv().await {
        let user_id = { session.lock().await.user_id };

        match event {
            Event::Close(user_id_to_close) if user_id_to_close == user_id => break,
            Event::Close(_) => continue,
            Event::Data {
                response,
                share_with,
            } => {
                let msg = serde_json::json!(response).to_string();

                let is_should_send = match share_with {
                    EventShare::One(id) => user_id == id,
                    EventShare::Many(users) => users.contains(&user_id),
                };

                if is_should_send {
                    let _ = sender.send(msg.into()).await;
                }
            }
        }
    }

    debug!(
        "Writer was closed for user {:?}",
        session.lock().await.user_id
    );
}
