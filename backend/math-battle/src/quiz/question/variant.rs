use std::str::FromStr;

use uuid::Uuid;

use crate::QuestionId;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Variant {
    id: VariantId,
    question_id: QuestionId,
    value: f32,
}

impl Variant {
    #[inline]
    pub fn new(question_id: QuestionId, value: f32) -> Self {
        let id = VariantId::random();

        Self {
            question_id,
            id,
            value,
        }
    }

    #[inline]
    pub fn question_id(&self) -> QuestionId {
        self.question_id
    }

    #[inline]
    pub fn id(&self) -> VariantId {
        self.id
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariantId(Uuid);

impl Default for VariantId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

impl FromStr for VariantId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<&str> for VariantId {
    type Error = uuid::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Uuid::from_str(s).map(Self)
    }
}

impl ToString for VariantId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl VariantId {
    #[inline]
    pub fn random() -> Self {
        Self::default()
    }

    #[inline]
    pub fn value(&self) -> Uuid {
        self.0
    }
}
