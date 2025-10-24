use std::time::Duration;

use thiserror::Error;
use tokio::time::sleep;

const ONE_SECOND: Duration = Duration::from_secs(1);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timer {
    timer: Duration,
}

impl Timer {
    pub fn new(time_limit: TimeLimit) -> Self {
        let timer = time_limit.duration();

        Self { timer }
    }

    #[inline]
    pub fn timer(&self) -> Duration {
        self.timer
    }

    pub async fn tick(&mut self) -> Option<Duration> {
        if self.timer.is_zero() {
            return None;
        }

        sleep(ONE_SECOND).await;
        self.timer = self.timer.saturating_sub(ONE_SECOND);

        Some(self.timer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeLimit(Duration);

impl Default for TimeLimit {
    fn default() -> Self {
        Self(Self::DEFAULT_TIME_LIMIT)
    }
}

impl TimeLimit {
    pub const MIN_TIME_LIMIT: Duration = Duration::from_secs(3);
    pub const MAX_TIME_LIMIT: Duration = Duration::from_secs(20);
    pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(15);

    pub fn new(time_limit: Duration) -> Result<Self, TimeLimitError> {
        match time_limit {
            x if x < Self::MIN_TIME_LIMIT => Err(TimeLimitError::TooLess),
            x if x > Self::MAX_TIME_LIMIT => Err(TimeLimitError::TooMuch),
            _ => Ok(Self(time_limit)),
        }
    }

    #[inline]
    pub const fn duration(&self) -> Duration {
        self.0
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeLimitError {
    #[error("time limit must be more than {:?}", TimeLimit::MIN_TIME_LIMIT)]
    TooLess,
    #[error("time limit must be less than {:?}", TimeLimit::MAX_TIME_LIMIT)]
    TooMuch,
}
