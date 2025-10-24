mod leaderboard;
mod lobby_id;
mod round;

pub use leaderboard::{Leaderboard, Member, Score};
pub use lobby_id::LobbyId;

#[derive(Debug, Clone)]
pub struct Lobby {
    id: LobbyId,
    leaderboard: Leaderboard,
}
