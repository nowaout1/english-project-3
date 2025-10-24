use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use thiserror::Error;
use tokio::{
    sync::{
        Mutex,
        broadcast::{self, Receiver, Sender, error::SendError},
    },
    task::{JoinHandle, spawn},
};

use crate::{room::ROOM_CAPACITY, user::User, user::UserId};

mod leaderboard;
mod question;
mod quiz_id;
mod timer;

pub use leaderboard::{Leaderboard, Participant, Score};
pub use question::{
    Complexity, Expression, ExpressionConfig, ExpressionError, Operation, OperationError, Question,
    QuestionError, QuestionId, Variant, VariantId,
};
pub use quiz_id::QuizId;
pub use timer::{TimeLimit, TimeLimitError, Timer};

const PARTICIPANTS_COUNT: usize = ROOM_CAPACITY;

pub type QuizParticipant = (Sender<ParticipantEvent>, Receiver<LeadingEvent>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuizConfig {
    questions_total: u8,
    time_limit: TimeLimit,
    complexity: Complexity,
    operations: Arc<[Operation]>,
}

impl Default for QuizConfig {
    fn default() -> Self {
        Self {
            questions_total: 4,
            time_limit: TimeLimit::default(),
            complexity: Complexity::Medium,
            operations: Arc::new(Operation::all()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParticipantEvent {
    QuestionRequested,
    TimeRequested,
    LeaderboardRequested,
    AnswerSubmitted(AnswerSubmitted),
    ParticipantReturned(UserId),
    ParticipantLeft(UserId),
}

#[derive(Debug, Clone)]
pub struct AnswerSubmitted {
    pub user_id: UserId,
    pub question_id: QuestionId,
    pub variant_id: VariantId,
}

#[derive(Debug, Clone)]
pub enum LeadingEvent {
    QuestionReady(Question),
    TimeUpdate(Duration),
    LeaderboardUpdated(Vec<Participant>),
    AnswerOutcome(AnswerOutcome),
    ScoreUpdated(ScoreUpdated),
    ParticipantReturned(UserId),
    ParticipantLeft(UserId),
    Finished,
}

#[derive(Debug, Clone)]
pub enum AnswerOutcome {
    AnswerEvaluated(AnswerEvaluated),
    AlreadyAnswered(UserId),
    NotRelevant(UserId),
}

#[derive(Debug, Clone)]
pub struct AnswerEvaluated {
    pub user_id: UserId,
    pub is_correct: bool,
    pub submitted_variant_id: VariantId,
    pub correct_variant_id: VariantId,
}

#[derive(Debug, Clone)]
pub struct ScoreUpdated {
    pub user_id: UserId,
    pub current_score: Score,
    pub previous_score: Score,
    pub points_earned: Score,
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuizError {
    #[error("can't start another quiz while one is in progress")]
    AlreadyStarted,
    #[error("only one answer per user")]
    AlreadyAnswered,
    #[error("question exception occurred")]
    Question(QuestionError),
}

#[derive(Debug)]
pub struct Quiz {
    id: QuizId,
    cfg: QuizConfig,
    leading: Option<JoinHandle<()>>,
    participant: Option<QuizParticipant>,
}

impl Clone for Quiz {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            cfg: self.cfg.clone(),
            leading: None,
            participant: None,
        }
    }
}

impl Drop for Quiz {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Quiz {
    pub fn new(cfg: QuizConfig) -> Self {
        let id = QuizId::random();
        let leading = None;
        let participant = None;

        Self {
            id,
            cfg,
            leading,
            participant,
        }
    }

    #[inline]
    pub fn id(&self) -> QuizId {
        self.id
    }

    #[inline]
    pub fn is_already_started(&self) -> bool {
        self.leading.is_some()
    }

    pub fn start<'a, I>(&mut self, users: I) -> Result<QuizParticipant, QuizError>
    where
        I: Iterator<Item = &'a User> + Clone,
    {
        if self.is_already_started() {
            return Err(QuizError::AlreadyStarted);
        }

        let state = Arc::new(Mutex::new(QuizState::new(users, &self.cfg)));

        let (leading_tx, leading_rx) = broadcast::channel::<LeadingEvent>(10);
        let (participant_tx, participant_rx) = broadcast::channel::<ParticipantEvent>(10);

        let cfg = self.cfg.clone();

        self.leading = Some(spawn(async move {
            let _ = tokio::join!(
                Self::leading(leading_tx.clone(), &cfg, Arc::clone(&state)),
                Self::helper(leading_tx, participant_rx, state)
            );
        }));

        self.participant = Some((participant_tx.clone(), leading_rx.resubscribe()));

        Ok((participant_tx, leading_rx))
    }

    pub async fn stop(&mut self) {
        if let Some(quiz) = &self.leading {
            quiz.abort();
        }
        self.leading = None;
    }

    pub fn participant_request(&self, event: ParticipantEvent) {
        if let Some((tx, _)) = &self.participant {
            let _ = tx.send(event);
        }
    }

    async fn leading(
        tx: Sender<LeadingEvent>,
        cfg: &QuizConfig,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        for _ in 1..=cfg.questions_total {
            Self::question(tx.clone(), &cfg, Arc::clone(&state)).await?;

            let mut timer = Timer::new(cfg.time_limit);

            while let Some(time_left) = timer.tick().await {
                if { state.lock().await.participants_count() } == 0 {
                    break;
                }

                if Self::is_all_answered(Arc::clone(&state)).await {
                    break;
                }

                let _ = tx.send(LeadingEvent::TimeUpdate(time_left));
                state.lock().await.update_remaining_time(time_left);
            }

            Self::update_leaderboard(tx.clone(), Arc::clone(&state)).await?;
        }

        let _ = tx.send(LeadingEvent::Finished);

        Ok(())
    }

    async fn question(
        tx: Sender<LeadingEvent>,
        cfg: &QuizConfig,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        let question = {
            let question =
                Question::new(cfg.complexity, &cfg.operations).expect("failed to create question");

            let mut state = state.lock().await;
            state.update_question(question.clone());

            question
        };

        let _ = tx.send(LeadingEvent::QuestionReady(question));

        Ok(())
    }

    async fn is_all_answered(state: Arc<Mutex<QuizState>>) -> bool {
        let state = state.lock().await;
        state.answers_total() == state.participants_count()
    }

    async fn update_leaderboard(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        let mut state = state.lock().await;
        let scores = state.update_leaderboard();

        for score in scores {
            tx.send(LeadingEvent::ScoreUpdated(score))?;
        }

        let _ = tx.send(LeadingEvent::LeaderboardUpdated(state.leaderboard()));

        Ok(())
    }

    async fn helper(
        tx: Sender<LeadingEvent>,
        mut rx: Receiver<ParticipantEvent>,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        while let Ok(event) = rx.recv().await {
            let tx = tx.clone();
            let state = Arc::clone(&state);

            match event {
                ParticipantEvent::QuestionRequested => Self::handle_get_question(tx, state).await?,
                ParticipantEvent::TimeRequested => {
                    Self::handle_get_remaining_time(tx, state).await?
                }
                ParticipantEvent::LeaderboardRequested => {
                    Self::handle_get_leaderboard(tx, state).await?
                }
                ParticipantEvent::AnswerSubmitted(answer) => {
                    Self::handle_check_answer(tx, state, answer).await?
                }
                ParticipantEvent::ParticipantReturned(user_id) => {
                    Self::handle_returned(tx, state, user_id).await?
                }
                ParticipantEvent::ParticipantLeft(user_id) => {
                    Self::handle_left(tx, state, user_id).await?
                }
            }
        }

        Ok(())
    }

    async fn handle_get_question(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        let state = state.lock().await;
        let question = state.question();

        let _ = tx.send(LeadingEvent::QuestionReady(question.clone()));

        Ok(())
    }

    async fn handle_get_remaining_time(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        let state = state.lock().await;
        let remaining_time = state.remaining_time();

        let _ = tx.send(LeadingEvent::TimeUpdate(remaining_time));

        Ok(())
    }

    async fn handle_get_leaderboard(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
    ) -> Result<(), SendError<LeadingEvent>> {
        let mut state = state.lock().await;
        let leaderboard = state.leaderboard();

        let _ = tx.send(LeadingEvent::LeaderboardUpdated(leaderboard));

        Ok(())
    }

    async fn handle_check_answer(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
        answer: AnswerSubmitted,
    ) -> Result<(), SendError<LeadingEvent>> {
        let mut state = state.lock().await;

        if state.participant_ids.contains(&answer.user_id) {
            let user_id = answer.user_id;

            let response = match state.check_answer(answer) {
                Ok(result) => LeadingEvent::AnswerOutcome(AnswerOutcome::AnswerEvaluated(result)),
                Err(QuizError::AlreadyAnswered) => {
                    LeadingEvent::AnswerOutcome(AnswerOutcome::AlreadyAnswered(user_id))
                }
                Err(QuizError::Question(QuestionError::NotRelevant)) => {
                    LeadingEvent::AnswerOutcome(AnswerOutcome::NotRelevant(user_id))
                }
                Err(error) => {
                    eprintln!("got unexpected result of variant check {error:?}");
                    return Ok(());
                }
            };

            let _ = tx.send(response);
        } else {
            // TODO: handle answer from non-participant of quiz
        }

        Ok(())
    }

    async fn handle_returned(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
        user_id: UserId,
    ) -> Result<(), SendError<LeadingEvent>> {
        let mut state = state.lock().await;
        state.participant_returned(user_id);

        let _ = tx.send(LeadingEvent::ParticipantReturned(user_id));

        Ok(())
    }

    async fn handle_left(
        tx: Sender<LeadingEvent>,
        state: Arc<Mutex<QuizState>>,
        user_id: UserId,
    ) -> Result<(), SendError<LeadingEvent>> {
        let mut state = state.lock().await;
        let participants_count = state.participant_left(&user_id);

        let _ = tx.send(LeadingEvent::ParticipantLeft(user_id));

        if participants_count == 0 {
            let _ = tx.send(LeadingEvent::Finished);
        }

        Ok(())
    }
}

// TODO: better entity
type IsCorrect = bool;

#[derive(Debug, Clone)]
struct QuizState<const N: usize = PARTICIPANTS_COUNT> {
    // TODO: merge users identifiers and question in one entity
    answers: HashMap<UserId, IsCorrect>,
    question: Question,

    remaining_time: Duration,
    leaderboard: Leaderboard<N>,
    participant_ids: HashSet<UserId>,
}

impl QuizState {
    pub fn new<'a, I>(users: I, cfg: &QuizConfig) -> Self
    where
        I: Iterator<Item = &'a User> + Clone,
    {
        let question =
            Question::new(cfg.complexity, &cfg.operations).expect("failed to create question");
        let remaining_time = cfg.time_limit.duration();
        let answers = HashMap::new();
        let participant_ids = users.clone().into_iter().map(User::id).collect();
        let leaderboard = Leaderboard::from(users);

        Self {
            question,
            remaining_time,
            leaderboard,
            answers,
            participant_ids,
        }
    }

    #[inline]
    pub fn question(&self) -> &Question {
        &self.question
    }

    #[inline]
    pub fn remaining_time(&self) -> Duration {
        self.remaining_time
    }

    #[inline]
    pub fn leaderboard(&mut self) -> Vec<Participant> {
        self.leaderboard.leaderboard().cloned().collect()
    }

    pub fn update_leaderboard(&mut self) -> Vec<ScoreUpdated> {
        const POINTS_PER_QUESTION: u16 = 10;

        let correct_count = self
            .answers
            .values()
            .filter(|&&is_correct| is_correct)
            .count();

        POINTS_PER_QUESTION
            .checked_div(correct_count as _)
            .map(Score::new)
            .map(|points| {
                self.answers
                    .iter()
                    .filter_map(|(user_id, &is_correct)| is_correct.then_some(user_id))
                    .map(|&user_id| {
                        let previous_score =
                            self.leaderboard.score(&user_id).expect("user not found");

                        let current_score = self
                            .leaderboard
                            .add_score(user_id, points)
                            .expect("failed to award points to user");

                        ScoreUpdated {
                            user_id,
                            previous_score,
                            current_score,
                            points_earned: points,
                        }
                    })
                    .collect::<Vec<ScoreUpdated>>()
            })
            .unwrap_or_default()
    }

    pub fn check_answer(
        &mut self,
        AnswerSubmitted {
            user_id,
            question_id,
            variant_id,
        }: AnswerSubmitted,
    ) -> Result<AnswerEvaluated, QuizError> {
        let is_correct = self
            .question
            .check_answer(question_id, variant_id)
            .map_err(QuizError::Question)?;

        self.add_user_answer(user_id, is_correct)?;

        let answer_evaluated = AnswerEvaluated {
            user_id,
            is_correct,
            submitted_variant_id: variant_id,
            correct_variant_id: self.question.correct_variant_id(),
        };

        Ok(answer_evaluated)
    }

    #[inline]
    pub fn answers_total(&self) -> usize {
        self.answers.len()
    }

    #[inline]
    pub fn participants_count(&self) -> usize {
        self.participant_ids.len()
    }

    #[inline]
    pub fn participant_returned(&mut self, user_id: UserId) {
        self.participant_ids.insert(user_id);
    }

    #[inline]
    pub fn participant_left(&mut self, user_id: &UserId) -> usize {
        self.participant_ids.remove(user_id);
        self.participants_count()
    }

    #[inline]
    pub fn update_remaining_time(&mut self, remaining_time: Duration) {
        self.remaining_time = remaining_time;
    }

    #[inline]
    pub fn update_question(&mut self, question: Question) {
        self.question = question;
        self.answers.clear();
    }

    fn add_user_answer(&mut self, user_id: UserId, is_correct: IsCorrect) -> Result<(), QuizError> {
        if self.answers.contains_key(&user_id) {
            return Err(QuizError::AlreadyAnswered);
        }

        self.answers.insert(user_id, is_correct);

        Ok(())
    }
}
