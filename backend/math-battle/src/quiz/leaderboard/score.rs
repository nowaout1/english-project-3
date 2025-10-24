use std::ops::{Add, AddAssign};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(u16);

impl Score {
    pub const MIN_SCORE: Self = Self::new(0);
    pub const MAX_SCORE: Self = Self::new(100);
    pub const SCORE_STEP: Self = Self::new(Self::MAX_SCORE.value() / 10);

    #[inline]
    pub const fn new(score: u16) -> Self {
        Self(score)
    }

    #[inline]
    pub const fn value(&self) -> u16 {
        self.0
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let result = {
            let min = Self::MIN_SCORE.value();
            let max = Self::MAX_SCORE.value();
            let result = self.0 + rhs.0;

            result.clamp(min, max)
        };

        Self(result)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}
