use std::fmt::Display;
use std::ops::Range;
use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

use rand::seq::IndexedRandom;
use thiserror::Error;

const NUMERIC_SYSTEM: f32 = 10_f32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionConfig<'a, T>
where
    T: Clone + Copy,
{
    pub operations: &'a [Operation],
    pub fractional_count: u8,
    pub range: Range<T>,
    pub max_depth: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression<T> {
    Leaf(T),
    Branch {
        a: Arc<Self>,
        b: Arc<Self>,
        operation: Operation,
        result: T,
    },
}

impl<T> Expression<T>
where
    T: RandomByRange<T>
        + Clone
        + Copy
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Into<f32>
        + FloatAsInt<f32>,
{
    pub fn random(cfg: ExpressionConfig<T>) -> Result<Self, ExpressionError> {
        if cfg.max_depth == 0 {
            let range = cfg.range.clone();
            let fractional_count = cfg.fractional_count;
            let number = T::random_range(range, fractional_count);

            return Ok(Self::Leaf(number));
        }

        loop {
            let cfg = ExpressionConfig {
                max_depth: rand::random_range(0..cfg.max_depth),
                ..cfg.clone()
            };

            let a: Expression<T> = Self::random(cfg.clone())?;
            let b: Expression<T> = Self::random(cfg.clone())?;

            let operation =
                Operation::from_array(cfg.operations).map_err(ExpressionError::InvalidOperation)?;

            let result = {
                let a: f32 = a.clone().into();
                let b: f32 = b.clone().into();

                match operation {
                    Operation::Add => a + b,
                    Operation::Sub => a - b,
                    Operation::Mul => a * b,
                    Operation::Div => a / b,
                }
            };

            if {
                let is_normal = result.is_normal();
                let is_fixed = result == to_fixed(result, cfg.fractional_count);
                is_normal && is_fixed
            } {
                if let Ok(result) = unsafe { T::as_int(result) } {
                    break Ok(Self::Branch {
                        a: Arc::new(a),
                        b: Arc::new(b),
                        result,
                        operation,
                    });
                }
            }
        }
    }
}

impl<T> Expression<T>
where
    T: Clone + Copy,
{
    pub fn result(&self) -> &T {
        match self {
            Self::Leaf(result) => result,
            Self::Branch { result, .. } => result,
        }
    }
}

impl<T> ToString for Expression<T>
where
    T: Display,
{
    fn to_string(&self) -> String {
        match self {
            Self::Leaf(x) => x.to_string(),
            Self::Branch {
                a, b, operation, ..
            } => {
                let a = {
                    let a_string = a.to_string();

                    match **a {
                        Self::Branch { .. } => format!("({a_string})"),
                        Self::Leaf(_) => a_string,
                    }
                };

                let b = {
                    let b_string = b.to_string();

                    match **b {
                        Self::Branch { .. } => format!("({b_string})"),
                        Self::Leaf(_) => b_string,
                    }
                };

                let operation = operation.to_string();

                format!("{a} {operation} {b}")
            }
        }
    }
}

impl<T> From<Expression<T>> for f32
where
    T: Clone + Copy + Into<f32>,
{
    fn from(value: Expression<T>) -> Self {
        value.result().to_owned().into()
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionError {
    #[error("got invalid operation")]
    InvalidOperation(OperationError),
}

pub trait RandomByRange<T> {
    fn random_range(range: Range<T>, fix: u8) -> Self;
}

macro_rules! impl_random_for_float {
    ($Float:ty) => {
        impl RandomByRange<$Float> for $Float {
            fn random_range(range: Range<$Float>, fix: u8) -> Self {
                let n = rand::random_range(range);
                to_fixed(n, fix)
            }
        }
    };
}

impl_random_for_float!(f32);

pub trait FloatAsInt<F>: Sized {
    unsafe fn as_int(float: F) -> Result<Self, FloatAsIntError>;
}

macro_rules! impl_float_to_int {
    ($Int:ty, $Float:ty) => {
        impl FloatAsInt<$Float> for $Int {
            unsafe fn as_int(float: $Float) -> Result<Self, FloatAsIntError> {
                let int = float as $Int;
                match int as $Float == float {
                    true => Ok(int),
                    false => Err(FloatAsIntError::Overflow),
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloatAsIntError {
    Overflow,
}

impl_float_to_int!(i16, f32);
impl_float_to_int!(i8, f32);
impl_float_to_int!(u16, f32);
impl_float_to_int!(u8, f32);
impl_float_to_int!(f32, f32);

pub fn to_fixed<T>(n: T, fix: u8) -> f32
where
    T: Mul + Div + Into<f32>,
{
    let fix = NUMERIC_SYSTEM.powf(fix as _);
    let fixed = (n.into() * fix).round() / fix;
    fixed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    #[inline]
    pub const fn all() -> [Self; 4] {
        [Self::Add, Self::Sub, Self::Mul, Self::Div]
    }

    pub fn from_array(operations: &[Operation]) -> Result<Self, OperationError> {
        operations
            .choose(&mut rand::rng())
            .ok_or(OperationError::NoOperations)
            .cloned()
    }
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        match self {
            Self::Add => "+".into(),
            Self::Sub => "-".into(),
            Self::Mul => "*".into(),
            Self::Div => "/".into(),
        }
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationError {
    #[error("no operations was provided")]
    NoOperations,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_to_string_flat() {
        let expr = {
            let a = Arc::new(Expression::Leaf(1));
            let b = Arc::new(Expression::Leaf(2));
            let operation = Operation::Add;
            let result = a.result() + b.result();

            Expression::Branch {
                a,
                b,
                operation,
                result,
            }
        };

        assert_eq!("1 + 2".to_string(), expr.to_string());
    }

    #[test]
    fn expression_to_string_nest() {
        let expr = {
            let a = {
                let a = Arc::new(Expression::Leaf(1));
                let b = Arc::new(Expression::Leaf(2));
                let operation = Operation::Add;
                let result = a.result() + b.result();

                Arc::new(Expression::Branch {
                    a,
                    b,
                    operation,
                    result,
                })
            };
            let b = Arc::new(Expression::Leaf(3));
            let operation = Operation::Mul;
            let result = a.result() + b.result();

            Expression::Branch {
                a,
                b,
                operation,
                result,
            }
        };

        assert_eq!("(1 + 2) * 3".to_string(), expr.to_string());
    }
}
