use thiserror::Error;

mod expression;
mod question_id;
mod variant;

pub use expression::{Expression, ExpressionConfig, ExpressionError, Operation, OperationError};
pub use question_id::QuestionId;
pub use variant::{Variant, VariantId};

pub const VARIANTS_COUNT: usize = 4;

#[derive(Debug, Clone)]
pub struct Question {
    id: QuestionId,
    expression: Expression<f32>,
    variants: [Variant; VARIANTS_COUNT],
    correct_variant_id: VariantId,
}

impl Question {
    pub fn new(complexity: Complexity, operations: &[Operation]) -> Result<Self, QuestionError> {
        let expression = match complexity {
            Complexity::Easy => Self::easy_expression(operations),
            Complexity::Medium => Self::medium_expression(operations),
            Complexity::Hard => Self::hard_expression(operations),
        }?;

        let question_id = QuestionId::random();

        let (variants, correct_variant_id) = {
            let correct_result = *expression.result();

            let mut variants = [
                Variant::new(question_id, correct_result + rand::random::<i8>() as f32),
                Variant::new(question_id, correct_result + rand::random::<i8>() as f32),
                Variant::new(question_id, correct_result + rand::random::<u8>() as f32),
                Variant::new(question_id, correct_result + rand::random::<u8>() as f32),
            ];

            let correct_variant_id = {
                let random_idx = rand::random_range(0..variants.len());
                let random_variant = &mut variants[random_idx];

                *random_variant = Variant::new(question_id, correct_result);

                random_variant.id()
            };

            (variants, correct_variant_id)
        };

        Ok(Self {
            id: question_id,
            expression,
            variants,
            correct_variant_id,
        })
    }

    pub fn check_answer(
        &self,
        question_id: QuestionId,
        variant_id: VariantId,
    ) -> Result<bool, QuestionError> {
        if question_id != self.id {
            return Err(QuestionError::NotRelevant);
        }

        Ok(self.correct_variant_id == variant_id)
    }

    #[inline]
    pub fn id(&self) -> QuestionId {
        self.id
    }

    #[inline]
    pub fn expression(&self) -> &Expression<f32> {
        &self.expression
    }

    #[inline]
    pub fn variants(&self) -> &[Variant; VARIANTS_COUNT] {
        &self.variants
    }

    #[inline]
    pub fn correct_variant_id(&self) -> VariantId {
        self.correct_variant_id
    }

    #[inline]
    fn easy_expression(operations: &[Operation]) -> Result<Expression<f32>, QuestionError> {
        Expression::random(ExpressionConfig {
            max_depth: 1,
            fractional_count: 0,
            range: 0_f32..100_f32,
            operations,
        })
        .map_err(QuestionError::ExpressionError)
    }

    #[inline]
    fn medium_expression(operations: &[Operation]) -> Result<Expression<f32>, QuestionError> {
        Expression::random(ExpressionConfig {
            max_depth: 2,
            fractional_count: 0,
            range: -5_f32..15_f32,
            operations,
        })
        .map_err(QuestionError::ExpressionError)
    }

    #[inline]
    fn hard_expression(operations: &[Operation]) -> Result<Expression<f32>, QuestionError> {
        Expression::random(ExpressionConfig {
            max_depth: 3,
            fractional_count: 1,
            range: -20_f32..50_f32,
            operations,
        })
        .map_err(QuestionError::ExpressionError)
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuestionError {
    #[error("question is not relevant anymore")]
    NotRelevant,
    #[error("got expression error")]
    ExpressionError(ExpressionError),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Complexity {
    #[default]
    Easy,
    Medium,
    Hard,
}
