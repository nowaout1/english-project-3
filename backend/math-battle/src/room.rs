use std::collections::HashSet;

use thiserror::Error;

use crate::user::UserId;

mod room_id;

pub use room_id::RoomId;

pub const ROOM_CAPACITY: usize = 50;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoomError {
    #[error("room is full")]
    Crowded,
}

#[derive(Debug, Clone)]
pub struct Room {
    id: RoomId,
    owner_id: UserId,
    members_ids: HashSet<UserId>,
}

impl Room {
    #[inline]
    pub fn new(owner_id: UserId) -> Self {
        let id = RoomId::random();
        let users = HashSet::from([owner_id]);

        Self {
            id,
            owner_id,
            members_ids: users,
        }
    }

    #[inline]
    pub fn id(&self) -> RoomId {
        self.id
    }

    #[inline]
    pub fn owner_id(&self) -> UserId {
        self.owner_id
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.members_ids.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.members_ids.is_empty()
    }

    pub fn members_ids(&self) -> Vec<UserId> {
        self.members_ids.iter().cloned().collect()
    }

    pub fn add_member(&mut self, member_id: UserId) -> Result<(), RoomError> {
        if self.count() >= ROOM_CAPACITY {
            return Err(RoomError::Crowded);
        }

        self.members_ids.insert(member_id);

        Ok(())
    }

    pub fn remove_member(&mut self, member_id: &UserId) {
        self.members_ids.remove(member_id);
    }
}
