use super::Score;
use crate::user::{UserId, Username};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Participant {
    id: UserId,
    username: Username,
    score: Score,
}

impl Participant {
    pub fn new(id: UserId, username: Username) -> Self {
        let score = Score::new(0);

        Self {
            id,
            username,
            score,
        }
    }

    #[inline]
    pub fn id(&self) -> UserId {
        self.id
    }

    #[inline]
    pub fn username(&self) -> &Username {
        &self.username
    }

    #[inline]
    pub fn score(&self) -> Score {
        self.score
    }

    #[inline]
    pub fn add_score(&mut self, score: Score) -> Score {
        self.score += score;
        self.score
    }
}
