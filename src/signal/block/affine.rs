use std::iter::Sum;

use crate::signal::SignalBlock;
use ndarray::ScalarOperand;
use thiserror::Error;

pub use num_traits::{Float, One, Zero};
pub use sci_rs::na::RealField;

#[derive(Error, Debug)]
pub enum AffineError {}

/// Signal processor that performs an affine transform of its input.
/// Typically used to remove known DC bias from a signal and scale it
/// into usable units.
///
/// Signal flow (per sample)
///
/// 1. Subtract "offset" from sample.
/// 2. Multiply result by "gain".
pub struct AffineTransform<T>
where
    T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand,
{
    offset: T,
    gain: T,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> SignalBlock<T>
    for AffineTransform<T>
{
    fn reset(&mut self) {}

    fn process(&mut self, input: &ndarray::Array1<T>) -> ndarray::Array1<T> {
        let result = (input - self.offset) * self.gain;
        result
    }
}

pub struct AffineTransformBuilder<T> {
    offset: Option<T>,
    gain: Option<T>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> Default
    for AffineTransformBuilder<T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> AffineTransformBuilder<T> {
    pub fn new() -> Self {
        Self {
            offset: None,
            gain: None,
        }
    }

    /// Subtract this value from every input sample.
    pub fn offset(mut self, offset: T) -> Self {
        self.offset.replace(offset);
        self
    }

    /// Multiply every sample by this value (after offset subtraction)
    pub fn gain(mut self, gain: T) -> Self {
        self.gain.replace(gain);
        self
    }

    /// Construct an affine transform.
    pub fn build(self) -> Result<AffineTransform<T>, AffineError> {
        let mut result = AffineTransform {
            offset: self.offset.unwrap_or(T::zero()),
            gain: self.gain.unwrap_or(T::one()),
        };
        result.reset();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::AffineTransformBuilder;

    #[test]
    fn test_one() {
        AffineTransformBuilder::new()
            .offset(15000 as f32)
            .gain(0.00004)
            .build()
            .expect("build");
    }
}
