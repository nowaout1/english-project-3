use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Username(String);

impl Username {
    const MAX_LENGTH: usize = 24;

    pub fn new(username: String) -> Result<Self, UsernameError> {
        if username.is_empty() {
            return Err(UsernameError::Empty);
        }

        if username.len() > Self::MAX_LENGTH {
            return Err(UsernameError::TooLong);
        }

        Ok(Self(username))
    }
}

impl ToString for Username {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsernameError {
    #[error("username must not be empty")]
    Empty,
    #[error("username is too long")]
    TooLong,
}
