use crate::{
    quiz::Score,
    user::{UserId, Username},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Member {
    id: UserId,
    username: Username,
    score: Score,
}

impl Member {
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
    pub fn score(&self) -> Score {
        self.score
    }
}
