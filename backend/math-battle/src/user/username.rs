use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Username(String);

impl Username {
    pub fn new(username: String) -> Result<Self, UsernameError> {
        if username.len() == 0 {
            return Err(UsernameError::Empty);
        }

        Ok(Self(username))
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UsernameError {
    #[error("username must not be empty")]
    Empty,
}
