use thiserror::Error;

mod user_id;
mod username;

pub use user_id::UserId;
pub use username::{Username, UsernameError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User {
    id: UserId,
    username: Username,
}

impl Default for User {
    fn default() -> Self {
        let id = UserId::random();
        let username = Username::new(format!("User")).expect("got invalid username");

        Self { id, username }
    }
}

impl User {
    pub fn new(username: String) -> Result<Self, MemberError> {
        let username = Username::new(username).map_err(MemberError::InvalidUsername)?;

        Ok(Self {
            username,
            ..Default::default()
        })
    }

    #[inline]
    pub fn id(&self) -> UserId {
        self.id
    }

    #[inline]
    pub fn username(&self) -> &Username {
        &self.username
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemberError {
    #[error("invalid username")]
    InvalidUsername(UsernameError),
}
