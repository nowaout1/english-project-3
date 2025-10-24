use crate::user::{User, UserId};

mod participant;
mod score;

pub use participant::Participant;
pub use score::Score;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Leaderboard<const N: usize> {
    participants: [Option<Participant>; N],
}

impl<'a, I, const N: usize> From<I> for Leaderboard<N>
where
    I: Iterator<Item = &'a User>,
{
    fn from(value: I) -> Self {
        let mut participants = [const { None }; _];

        for (i, user) in value.enumerate() {
            let participant = Participant::new(user.id(), user.username().clone());
            participants[i] = Some(participant);
        }

        Self {
            participants: participants,
        }
    }
}

impl<const N: usize> Leaderboard<N> {
    #[inline]
    pub fn count_of_participants(&self) -> usize {
        self.participants.iter().count()
    }

    pub fn leaderboard(&mut self) -> impl Iterator<Item = &Participant> {
        // TODO: interior mutability (`Mutex<T>` or smth idk)

        self.participants.sort();
        self.participants.reverse();
        self.participants.iter().filter_map(Into::into)
    }

    pub fn score(&self, user_id: &UserId) -> Option<Score> {
        self.participants
            .iter()
            .flatten()
            .find(|p| p.id() == *user_id)
            .map(|p| p.score())
    }

    pub fn add_score(&mut self, user_id: UserId, score: Score) -> Option<Score> {
        // TODO: refactor me

        let maybe_found = self.participants.iter_mut().find(|participant| {
            if let Some(participant) = participant {
                return participant.id() == user_id;
            }
            false
        });

        if let Some(Some(participant)) = maybe_found {
            return Some(participant.add_score(score));
        }

        None
    }
}
