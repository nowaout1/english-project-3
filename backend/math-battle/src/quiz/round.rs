use std::time::Duration;
use thiserror::Error;

mod expression;
mod round_id;
mod timeout;

pub use round_id::RoundId;
pub use timeout::{Timeout, TimeoutError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Round {
    id: RoundId,
    complexity: Complexity,
    timeout: Timeout,
}

impl Round {
    pub fn new(complexity: Complexity, timeout: Duration) -> Result<Self, RoundError> {
        let timeout = Timeout::new(timeout).map_err(RoundError::InvalidTimeout)?;
        let id = RoundId::random();

        Ok(Self {
            id,
            complexity,
            timeout,
        })
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoundError {
    #[error("got invalid timeout")]
    InvalidTimeout(TimeoutError),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Complexity {
    #[default]
    Easy,
    Medium,
    Hard,
}
