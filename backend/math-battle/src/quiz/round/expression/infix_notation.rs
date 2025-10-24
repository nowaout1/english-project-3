use std::ops::{Add, Div, Mul, Sub};

use rand::{distr::StandardUniform, seq::IteratorRandom};

use super::Operation;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfixNotation {
    a: f32,
    b: f32,
    result: f32,
    operation: Operation,
}

impl InfixNotation {
    pub fn random<F>(operations: &[Operation]) -> Self
    where
        F: Into<f32>,
        StandardUniform: rand::distr::Distribution<F>,
    {
        loop {
            let a: f32 = random_fixed_number::<F>(2);
            let b: f32 = random_fixed_number::<F>(2);

            let operation = operations
                .iter()
                .choose(&mut rand::rng())
                .expect("no operations provided")
                .to_owned();

            let result: f32 = match operation {
                Operation::Add => a + b,
                Operation::Sub => a - b,
                Operation::Mul => a * b,
                Operation::Div => a / b,
            };

            if result == to_fixed(result, 2) {
                break Self {
                    a,
                    b,
                    operation,
                    result,
                };
            }
        }
    }

    #[inline]
    pub fn a(&self) -> f32 {
        self.a
    }

    #[inline]
    pub fn b(&self) -> f32 {
        self.b
    }

    #[inline]
    pub fn result(&self) -> f32 {
        self.result
    }

    #[inline]
    pub fn operation(&self) -> Operation {
        self.operation
    }
}

impl Add for InfixNotation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            a: self.result,
            b: rhs.result,
            result: self.result + rhs.result,
            operation: Operation::Add,
        }
    }
}

impl Sub for InfixNotation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            a: self.result,
            b: rhs.result,
            result: self.result - rhs.result,
            operation: Operation::Sub,
        }
    }
}

impl Mul for InfixNotation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            a: self.result,
            b: rhs.result,
            result: self.result * rhs.result,
            operation: Operation::Mul,
        }
    }
}

impl Div for InfixNotation {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            a: self.result,
            b: rhs.result,
            result: self.result / rhs.result,
            operation: Operation::Mul,
        }
    }
}

fn random_fixed_number<F>(fix: u8) -> f32
where
    F: Into<f32>,
    StandardUniform: rand::distr::Distribution<F>,
{
    let n = rand::random::<F>().into();
    to_fixed(n, fix)
}

fn to_fixed<F>(n: F, fix: u8) -> f32
where
    F: Mul + Div + Into<f32>,
{
    let fix = 10_f32.powf(fix as _);
    let fixed = (n.into() * fix).round() / fix;
    fixed
}
