use std::time::Duration;

use thiserror::Error;

pub const MIN_TIMEOUT: Duration = Duration::from_secs(3);
pub const MAX_TIMEOUT: Duration = Duration::from_secs(20);
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timeout(Duration);

impl Default for Timeout {
    fn default() -> Self {
        Self(DEFAULT_TIMEOUT)
    }
}

impl Timeout {
    pub fn new(timeout: Duration) -> Result<Self, TimeoutError> {
        match timeout {
            x if x < MIN_TIMEOUT => Err(TimeoutError::TooLess),
            x if x > MAX_TIMEOUT => Err(TimeoutError::TooMuch),
            _ => Ok(Self(timeout)),
        }
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeoutError {
    #[error("timeout must be more than {MIN_TIMEOUT:?}")]
    TooLess,
    #[error("timeout must be less than {MAX_TIMEOUT:?}")]
    TooMuch,
}
