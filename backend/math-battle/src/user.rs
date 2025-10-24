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

impl From<UserId> for User {
    fn from(id: UserId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl Default for User {
    fn default() -> Self {
        let id = UserId::random();
        let username = {
            let id = id.to_string();
            let username = format!("User #{}", &id[id.len() - 4..]);
            Username::new(username).expect("got invalid username")
        };

        Self { id, username }
    }
}

impl User {
    pub fn new(username: String) -> Result<Self, UserError> {
        let username = Username::new(username).map_err(UserError::InvalidUsername)?;

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

    pub fn rename(&mut self, username: Username) {
        self.username = username;
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserError {
    #[error("invalid username")]
    InvalidUsername(UsernameError),
}
