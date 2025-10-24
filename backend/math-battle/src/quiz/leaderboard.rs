use crate::user::{UserId, Username};

mod member;
mod score;

pub use member::Member;
pub use score::Score;

#[derive(Debug, Default, Clone)]
pub struct Leaderboard {
    members: Vec<Member>,
}

impl Leaderboard {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.members.len()
    }

    #[inline]
    pub fn leaderboard(&mut self) -> impl Iterator<Item = &Member> {
        self.members.sort();
        self.members.reverse();
        self.members.iter()
    }

    #[inline]
    pub fn add_member(&mut self, id: UserId, username: Username) {
        let member = Member::new(id, username);
        self.members.push(member);
    }

    #[inline]
    pub fn remove_member(&mut self, id: UserId) {
        if let Some(index) = self.members.iter().position(|member| member.id() == id) {
            self.members.remove(index);
        }
    }
}
