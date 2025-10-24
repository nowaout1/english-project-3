use thiserror::Error;

use crate::user::{User, UserId};

mod room_id;

pub use room_id::RoomId;

pub const ROOM_CAPACITY: usize = 50;

#[derive(Debug, Default, Clone)]
pub struct Room {
    id: RoomId,
    users: Vec<User>,
}

impl Room {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn id(&self) -> RoomId {
        self.id
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.users.len()
    }

    #[inline]
    pub fn add_member(&mut self, user: User) -> Result<(), RoomError> {
        if self.count() >= ROOM_CAPACITY {
            return Err(RoomError::Crowded);
        }

        self.users.push(user);

        Ok(())
    }

    #[inline]
    pub fn remove_member(&mut self, id: UserId) {
        if let Some(idx) = self.users.iter().position(|user| user.id() == id) {
            self.users.remove(idx);
        }
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoomError {
    #[error("room is full")]
    Crowded,
}
