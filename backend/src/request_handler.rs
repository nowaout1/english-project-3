use std::{str::FromStr, sync::Arc};

use log::{debug, info};

use tokio::sync::{
    Mutex,
    broadcast::{Sender, error::SendError},
};

use math_battle::{
    AnswerOutcome, AnswerSubmitted, LeadingEvent, ParticipantEvent, QuestionId, QuizError,
    RoomError, UserId, Username, VariantId,
};

use crate::{
    Event, EventShare, Session,
    dto::{self, AnswerVariant, MessageKind, Participant, Response, ResponseData, ResponseError},
    state::{AppState, AppStateError},
};

type SendResult = Result<(), SendError<Event>>;

pub async fn room_create(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let user_id = { session.lock().await.user_id };
    let room_id = state.create_room(user_id).await.to_string();

    let event = Event::new(
        Response {
            kind: MessageKind::RoomCreate,
            data: Some(ResponseData::RoomCreated { room_id }),
            error: None,
        },
        EventShare::One(user_id),
    );

    tx.send(event)?;

    Ok(())
}

pub async fn room_join(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
    owner_id: String,
) -> SendResult {
    debug!("User requested to join room where owner id {owner_id:?}");

    // Check identifier
    let Ok(owner_id) = UserId::try_from(owner_id.as_str()) else {
        fail(
            tx,
            session,
            MessageKind::RoomJoin,
            ResponseError::invalid_id(),
        )
        .await?;
        return Ok(());
    };

    let user_id = { session.lock().await.user_id };

    // Try to join room
    if let Err(error) = state.join_room(&owner_id, user_id).await {
        match error {
            AppStateError::Room(RoomError::Crowded) => {
                fail(tx, session, MessageKind::RoomJoin, ResponseError::crowded()).await?
            }
            AppStateError::RoomNotFound => {
                fail(
                    tx,
                    session,
                    MessageKind::RoomJoin,
                    ResponseError::not_found(),
                )
                .await?
            }
            _ => unimplemented!(),
        };
        return Ok(());
    }

    // Update session info
    session.lock().await.room_id = Some(owner_id);

    info!("User joined to room where owner id {owner_id:?}");

    // Get members
    let Ok(members_ids) = state.get_room_member_ids(&owner_id).await else {
        fail(
            tx,
            session,
            MessageKind::RoomJoin,
            ResponseError::not_found(),
        )
        .await?;

        return Ok(());
    };

    // Get current user
    let user = state.get_user(&user_id).await.expect("user not found");

    info!("User {user_id:?} joined to room where owner id {owner_id:?}",);

    // Send status joined to user
    tx.send(Event::new(
        Response {
            kind: MessageKind::RoomJoin,
            data: None,
            error: None,
        },
        EventShare::One(user_id),
    ))?;

    // Notify members
    tx.send(Event::new(
        Response {
            kind: MessageKind::RoomMemberJoined,
            data: Some(ResponseData::RoomMemberJoined {
                member: dto::as_user_dto(user),
            }),
            error: None,
        },
        EventShare::Many(
            members_ids
                .iter()
                .filter(|&&member_id| member_id != user_id)
                .cloned()
                .collect(),
        ),
    ))?;

    Ok(())
}

pub async fn room_members_list(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    // Get room owner id
    let Some(owner_id) = session.lock().await.room_id else {
        fail(
            tx,
            session,
            MessageKind::RoomMembersList,
            ResponseError::not_found(),
        )
        .await?;

        return Ok(());
    };

    // Get members
    let Ok(members) = state.get_room_members(&owner_id).await else {
        fail(
            tx,
            session,
            MessageKind::RoomMembersList,
            ResponseError::not_found(),
        )
        .await?;

        return Ok(());
    };

    tx.send(Event::new(
        Response {
            kind: MessageKind::RoomMembersList,
            data: Some(ResponseData::RoomMembersList {
                members: members.into_iter().map(dto::as_user_dto).collect(),
            }),
            error: None,
        },
        EventShare::One(session.lock().await.user_id),
    ))?;

    Ok(())
}

pub async fn room_leave(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    // Get room owner id
    let Some(owner_id) = session.lock().await.room_id else {
        fail(
            tx,
            session,
            MessageKind::RoomLeave,
            ResponseError::not_found(),
        )
        .await?;

        return Ok(());
    };

    // Get members identifiers
    let members_ids = state.get_room_member_ids(&owner_id).await;

    // Try to leave room
    let (user_id, _) = {
        let mut session = session.lock().await;
        let user_id = session.user_id;

        // Update session info
        session.room_id = None;

        (user_id, state.leave_room(&owner_id, &user_id).await)
    };

    info!("User {user_id:?} leaved room where owner id {owner_id:?}");

    // Send status leaved to user
    tx.send(Event::new(
        Response {
            kind: MessageKind::RoomLeave,
            data: None,
            error: None,
        },
        EventShare::One(user_id),
    ))?;

    // Notify members
    if let Ok(members_ids) = members_ids {
        tx.send(Event::new(
            Response {
                kind: MessageKind::RoomMemberLeaved,
                data: Some(ResponseData::RoomMemberLeaved {
                    member_id: user_id.to_string(),
                }),
                error: None,
            },
            EventShare::Many(members_ids),
        ))?;
    }

    Ok(())
}

pub async fn user(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    if let Some(user) = state.get_user(&session.lock().await.user_id).await {
        tx.send(Event::new(
            Response {
                kind: MessageKind::User,
                data: Some(ResponseData::User {
                    user_id: user.id().to_string(),
                    username: user.username().to_string(),
                }),
                error: None,
            },
            EventShare::One(user.id()),
        ))?;
    }

    Ok(())
}

pub async fn username_updated(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
    username: String,
) -> SendResult {
    // Validate username
    let Ok(username) = Username::new(username) else {
        return fail(
            tx,
            session,
            MessageKind::UsernameUpdated,
            ResponseError::invalid_username(),
        )
        .await;
    };

    let (user_id, owner_id) = {
        let session = session.lock().await;
        let user_id = session.user_id;
        let owner_id = session.room_id;

        state.rename_user(&user_id, username.clone()).await;

        (user_id, owner_id)
    };

    info!("Username updated for user {user_id:?}");

    if let Some(owner_id) = owner_id
        && let Ok(members_ids) = state.get_room_member_ids(&owner_id).await
    {
        // Notify all room members
        tx.send(Event::new(
            Response {
                kind: MessageKind::UsernameUpdated,
                data: Some(ResponseData::UsernameUpdated {
                    user_id: user_id.to_string(),
                    username: username.to_string(),
                }),
                error: None,
            },
            EventShare::Many(members_ids),
        ))?;
    }

    Ok(())
}

pub async fn start_quiz(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let owner_id = {
        let session = session.lock().await;
        session.room_id
    };

    let Some(owner_id) = owner_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizStart,
            ResponseError::not_found(),
        )
        .await;
    };

    let Ok(members_ids) = state.get_room_member_ids(&owner_id).await else {
        return fail(
            tx,
            session,
            MessageKind::QuizStart,
            ResponseError::not_found(),
        )
        .await;
    };

    state.create_quiz(owner_id).await;

    let (_, mut quiz_rx) = {
        match state.start_quiz(&owner_id).await {
            Ok(x) => x,
            Err(AppStateError::Quiz(QuizError::AlreadyStarted)) => {
                return fail(
                    tx,
                    session,
                    MessageKind::QuizStart,
                    ResponseError::already_exists(),
                )
                .await;
            }
            Err(_) => {
                return fail(
                    tx,
                    session,
                    MessageKind::QuizStart,
                    ResponseError::not_found(),
                )
                .await;
            }
        }
    };

    {
        let mut state = state.clone();
        let tx = tx.clone();
        let members_ids = members_ids.clone();

        tokio::spawn(async move {
            info!("Quiz with owner id {:?} has started", owner_id);

            while let Ok(ev) = quiz_rx.recv().await {
                let members_ids = members_ids.clone();
                let event = match ev {
                    LeadingEvent::QuestionReady(question) => Event::new(
                        Response {
                            kind: MessageKind::QuizQuestion,
                            data: Some(ResponseData::QuizQuestion {
                                id: question.id().to_string(),
                                expression: question.expression().to_string(),
                                variants: question
                                    .variants()
                                    .map(|x| AnswerVariant {
                                        variant_id: x.id().to_string(),
                                        question_id: x.question_id().to_string(),
                                        value: x.value().to_string(),
                                    })
                                    .to_vec(),
                            }),
                            error: None,
                        },
                        EventShare::Many(members_ids),
                    ),
                    LeadingEvent::TimeUpdate(duration) => Event::new(
                        Response {
                            kind: MessageKind::QuizTimer,
                            data: Some(ResponseData::QuizTimer {
                                remaining_secs: duration.as_secs() as _,
                            }),
                            error: None,
                        },
                        EventShare::Many(members_ids),
                    ),
                    LeadingEvent::LeaderboardUpdated(participants) => Event::new(
                        Response {
                            kind: MessageKind::QuizLeaderboard,
                            data: Some(ResponseData::QuizLeaderboard {
                                participants: participants
                                    .into_iter()
                                    .map(|p| Participant {
                                        id: p.id().to_string(),
                                        username: p.username().to_string(),
                                        score: p.score().value(),
                                    })
                                    .collect(),
                            }),
                            error: None,
                        },
                        EventShare::Many(members_ids),
                    ),
                    LeadingEvent::AnswerOutcome(outcome) => match outcome {
                        AnswerOutcome::AlreadyAnswered(user_id) => Event::new(
                            Response {
                                kind: MessageKind::QuizCheck,
                                data: None,
                                error: Some(ResponseError::already_exists()),
                            },
                            EventShare::One(user_id),
                        ),
                        AnswerOutcome::NotRelevant(user_id) => Event::new(
                            Response {
                                kind: MessageKind::QuizCheck,
                                data: None,
                                error: Some(ResponseError::not_relevant()),
                            },
                            EventShare::One(user_id),
                        ),
                        AnswerOutcome::AnswerEvaluated(result) => Event::new(
                            Response {
                                kind: MessageKind::QuizCheck,
                                data: Some(ResponseData::QuizAnswerOutcome {
                                    user_id: result.user_id.to_string(),
                                    is_correct: result.is_correct,
                                    submitted_variant_id: result.submitted_variant_id.to_string(),
                                    correct_variant_id: result.correct_variant_id.to_string(),
                                }),
                                error: None,
                            },
                            EventShare::One(result.user_id),
                        ),
                    },
                    LeadingEvent::ScoreUpdated(ScoreUpdated) => continue,
                    LeadingEvent::ParticipantReturned(_) => continue,
                    LeadingEvent::ParticipantLeft(_) => continue,
                    LeadingEvent::Finished => {
                        state.remove_quiz(&owner_id).await;

                        let _ = tx.send(Event::new(
                            Response {
                                kind: MessageKind::QuizFinished,
                                data: None,
                                error: None,
                            },
                            EventShare::Many(members_ids),
                        ));

                        break;
                    }
                };

                let _ = tx.send(event);
            }

            info!("Quiz with owner id {:?} has finished", owner_id);
        });
    }

    // Notify room members
    tx.send(Event::new(
        Response {
            kind: MessageKind::QuizStart,
            data: Some(ResponseData::QuizStart {
                quiz_id: owner_id.to_string(),
            }),
            error: None,
        },
        EventShare::Many(members_ids),
    ))?;

    // Request for send first question
    quiz_question(tx, state, session).await?;

    Ok(())
}

pub async fn quiz_question(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let owner_id = {
        let session = session.lock().await;
        session.room_id
    };

    let Some(owner_id) = owner_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizQuestion,
            ResponseError::not_found(),
        )
        .await;
    };

    let _ = state
        .participant_request_quiz(&owner_id, ParticipantEvent::QuestionRequested)
        .await;

    Ok(())
}

pub async fn quiz_check(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
    question_id: String,
    variant_id: String,
) -> SendResult {
    let Some(owner_id) = session.lock().await.room_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizCheck,
            ResponseError::not_found(),
        )
        .await;
    };

    let Ok(question_id) = QuestionId::from_str(&question_id) else {
        return fail(
            tx,
            session,
            MessageKind::QuizCheck,
            ResponseError::invalid_id(),
        )
        .await;
    };

    let Ok(variant_id) = VariantId::from_str(&variant_id) else {
        return fail(
            tx,
            session,
            MessageKind::QuizCheck,
            ResponseError::not_found(),
        )
        .await;
    };

    let _ = state
        .participant_request_quiz(
            &owner_id,
            ParticipantEvent::AnswerSubmitted(AnswerSubmitted {
                user_id: session.lock().await.user_id,
                question_id,
                variant_id,
            }),
        )
        .await;

    Ok(())
}

pub async fn quiz_timer(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let Some(owner_id) = session.lock().await.room_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizTimer,
            ResponseError::not_found(),
        )
        .await;
    };

    let _ = state
        .participant_request_quiz(&owner_id, ParticipantEvent::TimeRequested)
        .await;

    Ok(())
}

pub async fn quiz_leaderboard(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let Some(owner_id) = session.lock().await.room_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizLeaderboard,
            ResponseError::not_found(),
        )
        .await;
    };

    let _ = state
        .participant_request_quiz(&owner_id, ParticipantEvent::LeaderboardRequested)
        .await;

    Ok(())
}

pub async fn quiz_leave(
    tx: Sender<Event>,
    state: &mut AppState,
    session: Arc<Mutex<Session>>,
) -> SendResult {
    let Some(owner_id) = session.lock().await.room_id else {
        return fail(
            tx,
            session,
            MessageKind::QuizStart,
            ResponseError::not_found(),
        )
        .await;
    };

    let _ = state
        .participant_request_quiz(
            &owner_id,
            ParticipantEvent::ParticipantLeft(session.lock().await.user_id),
        )
        .await;

    Ok(())
}

pub async fn fail(
    tx: Sender<Event>,
    session: Arc<Mutex<Session>>,
    kind: MessageKind,
    error: ResponseError,
) -> SendResult {
    tx.send(Event::new(
        Response {
            kind,
            data: None,
            error: Some(error),
        },
        EventShare::One(session.lock().await.user_id),
    ))?;

    Ok(())
}
