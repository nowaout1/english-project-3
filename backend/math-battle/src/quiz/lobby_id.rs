use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LobbyId(Uuid);

impl Default for LobbyId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl LobbyId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
