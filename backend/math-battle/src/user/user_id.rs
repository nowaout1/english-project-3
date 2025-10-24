use std::str::FromStr;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(Uuid);

impl Default for UserId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl FromStr for UserId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for UserId {
    type Error = uuid::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Uuid::from_str(s).map(Self)
    }
}

impl ToString for UserId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl UserId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
