#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Score(u16);

impl Score {
    #[inline]
    pub fn new(score: u16) -> Self {
        Self(score)
    }

    #[inline]
    pub fn value(&self) -> u16 {
        self.0
    }

    #[inline]
    pub fn add(&mut self, value: u16) {
        self.0 += value;
    }
}
