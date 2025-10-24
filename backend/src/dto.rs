use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Request {
    User,
    UsernameUpdated {
        username: String,
    },
    RoomCreate,
    RoomJoin {
        room_id: String,
    },
    RoomLeave,
    RoomMembersList,
    QuizStart,
    QuizJoin {
        room_id: String,
    },
    QuizQuestion,
    QuizCheck {
        question_id: String,
        variant_id: String,
    },
    QuizTimer,
    QuizLeaderboard,
    QuizLeave,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub struct Response {
    pub kind: MessageKind,
    pub data: Option<ResponseData>,
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ResponseData {
    User {
        user_id: String,
        username: String,
    },
    UsernameUpdated {
        user_id: String,
        username: String,
    },
    RoomCreated {
        room_id: String,
    },
    RoomMemberJoined {
        member: User,
    },
    RoomMemberLeaved {
        member_id: String,
    },
    RoomMembersList {
        members: Vec<User>,
    },
    QuizStart {
        quiz_id: String,
    },
    QuizQuestion {
        id: String,
        expression: String,
        variants: Vec<AnswerVariant>,
    },
    QuizTimer {
        remaining_secs: u16,
    },
    QuizAnswerOutcome {
        user_id: String,
        is_correct: bool,
        submitted_variant_id: String,
        correct_variant_id: String,
    },
    QuizLeaderboard {
        participants: Vec<Participant>,
    },
    QuizScore {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub struct ResponseError {
    code: String,
}

impl ResponseError {
    const INVALID_ID: &'static str = "INVALID_ID";
    const INVALID_USERNAME: &'static str = "INVALID_USERNAME";
    const NOT_FOUND: &'static str = "NOT_FOUND";
    const CROWDED: &'static str = "CROWDED";
    const NOT_RELEVANT: &'static str = "NOT_RELEVANT";
    const ALREADY_EXISTS: &'static str = "ALREADY_EXISTS";

    pub fn invalid_id() -> Self {
        Self {
            code: Self::INVALID_ID.into(),
        }
    }

    pub fn invalid_username() -> Self {
        Self {
            code: Self::INVALID_USERNAME.into(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: Self::NOT_FOUND.into(),
        }
    }

    pub fn crowded() -> Self {
        Self {
            code: Self::CROWDED.into(),
        }
    }

    pub fn not_relevant() -> Self {
        Self {
            code: Self::NOT_RELEVANT.into(),
        }
    }

    pub fn already_exists() -> Self {
        Self {
            code: Self::ALREADY_EXISTS.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    User,
    UsernameUpdated,
    RoomCreate,
    RoomJoin,
    RoomLeave,
    RoomMemberJoined,
    RoomMemberLeaved,
    RoomMembersList,
    QuizStart,
    QuizJoin,
    QuizLeave,
    QuizQuestion,
    QuizCheck,
    QuizTimer,
    QuizLeaderboard,
    QuizScore,
    QuizFinished,
}

pub fn as_user_dto(user: math_battle::User) -> User {
    User {
        id: user.id().to_string(),
        username: user.username().to_string(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Participant {
    pub id: String,
    pub username: String,
    pub score: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub struct AnswerVariant {
    pub variant_id: String,
    pub question_id: String,
    pub value: String,
}
