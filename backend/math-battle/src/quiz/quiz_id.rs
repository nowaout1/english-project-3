use std::str::FromStr;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuizId(Uuid);

impl Default for QuizId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl FromStr for QuizId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for QuizId {
    type Error = uuid::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Uuid::from_str(s).map(Self)
    }
}

impl ToString for QuizId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl QuizId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
