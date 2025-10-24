use rand::{
    Rng,
    distr::{Distribution, StandardUniform},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Distribution<Operation> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operation {
        match rng.random_range(0..4) {
            0 => Operation::Add,
            1 => Operation::Sub,
            2 => Operation::Mul,
            3 => Operation::Div,
            _ => unreachable!(),
        }
    }
}
