use axum::{Router, http::Method, response::IntoResponse, routing::post};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().expect("failed to initialize .env");

    let addr = dotenvy::var("ADDR").expect("environment variable `ADDR` must be set");

    let listener = TcpListener::bind(&addr).await?;
    let router = Router::new()
        .route("/api/v1/create_room", post(create_room))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::new().allow_methods([Method::GET, Method::POST])),
        );

    info!("Listening on: {addr}...");

    axum::serve(listener, router).await?;

    Ok(())
}

async fn create_room() -> impl IntoResponse {
    "hi"
}
