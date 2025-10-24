use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoundId(Uuid);

impl Default for RoundId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl RoundId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
