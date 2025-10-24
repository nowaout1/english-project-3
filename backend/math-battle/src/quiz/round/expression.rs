use std::ops::Range;

use rand::seq::IndexedRandom;

mod infix_notation;
mod operation;
mod set;

pub use infix_notation::InfixNotation;
pub use operation::Operation;
pub use set::Set;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Expression {
    nums: Vec<f32>,
    operations: Vec<Operation>,
}

impl Expression {
    pub fn new(cfg: Config) -> Self {
        let Config {
            operands_count,
            operations,
            set,
            limits,
        } = cfg;

        todo!()
    }
}

fn new_operand(set: Set) -> f32 {
    match set {
        Set::Natural => rand::random_range(0..i32::MAX) as _,
        Set::Integer => rand::random_range(-i32::MAX..i32::MAX) as _,
        Set::Rational => rand::random_range(-f32::MAX..f32::MAX),
    }
}

fn new_operation(operations: &[Operation]) -> Operation {
    operations
        .choose(&mut rand::rng())
        .expect("no operations provided")
        .clone()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    operands_count: u8,
    operations: Vec<Operation>,
    set: Set,
    limits: Range<i32>,
}
