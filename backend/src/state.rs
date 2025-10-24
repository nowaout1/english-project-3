use std::{collections::HashMap, sync::Arc};

use log::info;
use math_battle::{
    ParticipantEvent, Quiz, QuizConfig, QuizError, QuizParticipant, Room, RoomError, User, UserId,
    Username,
};
use tokio::sync::{Mutex, broadcast::Sender};

use crate::{Event, EventBus};

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppStateError {
    #[error("room error")]
    Room(RoomError),
    #[error("quiz error")]
    Quiz(QuizError),

    #[error("room not found")]
    RoomNotFound,
    #[error("quiz not found")]
    QuizNotFound,
}

pub type OwnerId = UserId;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    bus: EventBus<Event>,
    users: Arc<Mutex<HashMap<UserId, User>>>,
    rooms: Arc<Mutex<HashMap<OwnerId, Room>>>,
    quizzes: Arc<Mutex<HashMap<OwnerId, Quiz>>>,
}

impl AppState {
    pub fn tx(&self) -> Sender<Event> {
        self.bus.tx.clone()
    }

    pub async fn add_user(&mut self, user: User) -> UserId {
        let mut users = self.users.lock().await;
        let user_id = user.id();
        users.insert(user_id, user);
        info!("Added user with id {user_id:?}");
        info!("Current count of users: {}", users.len());
        user_id
    }

    pub async fn get_user(&mut self, user_id: &UserId) -> Option<User> {
        self.users.lock().await.get(user_id).cloned()
    }

    pub async fn rename_user(&mut self, user_id: &UserId, username: Username) {
        let mut users = self.users.lock().await;
        users.get_mut(user_id).map(|user| user.rename(username));
    }

    pub async fn remove_user(&mut self, user_id: &UserId) {
        let count = {
            let mut users = self.users.lock().await;
            users.remove(user_id);
            users.len()
        };

        info!("Removed user with id {user_id:?}");
        info!("Current count of users: {count}");
    }

    pub async fn get_room_member_ids(
        &self,
        owner_id: &OwnerId,
    ) -> Result<Vec<UserId>, AppStateError> {
        let member_ids = match self.rooms.lock().await.get(owner_id) {
            Some(room) => room.members_ids(),
            None => {
                return Err(AppStateError::RoomNotFound);
            }
        };

        Ok(member_ids)
    }

    pub async fn get_room_members(&self, owner_id: &OwnerId) -> Result<Vec<User>, AppStateError> {
        let member_ids = match self.rooms.lock().await.get(owner_id) {
            Some(room) => room.members_ids(),
            None => {
                return Err(AppStateError::RoomNotFound);
            }
        };

        let members = self
            .users
            .lock()
            .await
            .values()
            .filter(|user| member_ids.contains(&user.id()))
            .cloned()
            .collect::<Vec<User>>();

        Ok(members)
    }

    // Returns room id same as owner id
    pub async fn create_room(&mut self, owner_id: UserId) -> OwnerId {
        let mut rooms = self.rooms.lock().await;

        // If room exists then ignore
        if rooms.contains_key(&owner_id) {
            return owner_id;
        }

        let room = Room::new(owner_id);
        rooms.insert(owner_id, room);

        info!("Room created for owner {owner_id:?}");
        info!("Current count of rooms: {}", rooms.len());

        owner_id
    }

    pub async fn join_room(
        &mut self,
        owner_id: &OwnerId,
        user_id: UserId,
    ) -> Result<(), AppStateError> {
        let mut rooms = self.rooms.lock().await;

        let Some(room) = rooms.get_mut(owner_id) else {
            return Err(AppStateError::RoomNotFound);
        };

        room.add_member(user_id).map_err(AppStateError::Room)
    }

    pub async fn leave_room(
        &self,
        owner_id: &OwnerId,
        user_id: &UserId,
    ) -> Result<(), AppStateError> {
        let mut rooms = self.rooms.lock().await;

        let Some(room) = rooms.get_mut(owner_id) else {
            return Err(AppStateError::RoomNotFound);
        };

        room.remove_member(user_id);

        if room.is_empty() {
            rooms.remove(owner_id);

            info!("Removed room with owner id {owner_id:?}");
            info!("Current count of rooms: {}", rooms.len());
        }

        Ok(())
    }

    pub async fn create_quiz(&mut self, owner_id: UserId) -> OwnerId {
        let mut quizzes = self.quizzes.lock().await;

        if quizzes.contains_key(&owner_id) {
            return owner_id;
        }

        let quiz_cfg = QuizConfig::default();
        let quiz = Quiz::new(quiz_cfg);

        quizzes.insert(owner_id, quiz);

        info!("Quiz created where owner id {owner_id:?}");
        info!("Current count of quizzes: {}", quizzes.len());

        owner_id
    }

    pub async fn start_quiz(
        &mut self,
        owner_id: &OwnerId,
    ) -> Result<QuizParticipant, AppStateError> {
        let mut quizzes = self.quizzes.lock().await;

        let Some(quiz) = quizzes.get_mut(owner_id) else {
            return Err(AppStateError::QuizNotFound);
        };

        let Ok(room_members) = self.get_room_members(owner_id).await else {
            return Err(AppStateError::RoomNotFound);
        };

        quiz.start(room_members.iter()).map_err(AppStateError::Quiz)
    }

    pub async fn remove_quiz(&mut self, owner_id: &OwnerId) {
        let count = {
            let mut quizzes = self.quizzes.lock().await;
            quizzes.remove(owner_id);
            quizzes.len()
        };

        info!("Quiz removed where owner id {owner_id:?}");
        info!("Current count of quizzes: {count}");
    }

    pub async fn participant_request_quiz(
        &mut self,
        owner_id: &OwnerId,
        event: ParticipantEvent,
    ) -> Result<(), AppStateError> {
        let mut quizzes = self.quizzes.lock().await;

        let Some(quiz) = quizzes.get_mut(owner_id) else {
            return Err(AppStateError::QuizNotFound);
        };

        quiz.participant_request(event);

        Ok(())
    }
}
