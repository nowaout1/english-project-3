use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(Uuid);

impl Default for RoomId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl RoomId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
