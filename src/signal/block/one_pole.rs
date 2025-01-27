use std::iter::Sum;

use ndarray::ScalarOperand;
use num_traits::One;
use thiserror::Error;

pub use num_traits::{Float, Zero};
pub use sci_rs::na::RealField;

use crate::signal::SignalBlock;

use super::super::filter::lfilter::{lfilt_dyn, Ba};

#[derive(Clone, Copy)]
pub enum FilterType {
    LowPass,
    HighPass,
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::LowPass
    }
}

#[derive(Error, Debug)]
pub enum OnePoleError {
    #[error("Alpha is out of range (0-1)")]
    AlphaOutOfRange,
}

/// One-pole, low pass, "alpha/beta" filter.
///
/// A filter that decays its memory by some portion (alpha)
/// and incorporates the incoming sample by the remainder (beta).
///
/// Signal flow (per sample)
///
/// 1. Send through low-pass filter.
pub struct OnePoleLowPassFilter<T>
where
    T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand,
{
    taps: Ba<T>,
    memory: Ba<T>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> SignalBlock<T>
    for OnePoleLowPassFilter<T>
{
    fn reset(&mut self) {
        self.memory = self.taps.clone();
    }

    fn process(&mut self, input: &ndarray::Array1<T>) -> ndarray::Array1<T> {
        ndarray::Array1::from_iter(lfilt_dyn(input, &mut self.memory))
    }
}

pub struct OnePoleLowPassFilterBuilder<T> {
    alpha: Option<T>,
    filter_type: Option<FilterType>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> Default
    for OnePoleLowPassFilterBuilder<T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand>
    OnePoleLowPassFilterBuilder<T>
{
    pub fn new() -> Self {
        Self {
            alpha: None,
            filter_type: None,
        }
    }

    /// Configure as low-pass or high-pass.
    pub fn pass(mut self, t: FilterType) -> Self {
        self.filter_type.replace(t);
        self
    }

    /// Momentum coefficient for moving-average filter.
    /// (1.0 => infinitely stiff, never updates. 0.0 => follows every sample)
    pub fn alpha(mut self, alpha: T) -> Self {
        self.alpha.replace(alpha);
        self
    }

    /// Construct a filter block.
    pub fn build(self) -> Result<OnePoleLowPassFilter<T>, OnePoleError> {
        let alpha = self.alpha.unwrap_or(T::zero());
        if alpha < T::zero() || alpha > T::one() {
            return Err(OnePoleError::AlphaOutOfRange);
        }

        let b0: T;
        let b1: T;
        let a1: T;
        let neg = T::zero() - T::one();

        match self.filter_type.unwrap_or(FilterType::LowPass) {
            FilterType::LowPass => {
                b0 = T::one() - alpha;
                b1 = T::zero();
                a1 = neg * alpha;
            }
            FilterType::HighPass => {
                b0 = alpha;
                b1 = neg * alpha;
                a1 = neg * alpha;
            }
        }
        let b = [b0, b1];
        let zi0 = T::zero();
        let ba = Ba { b, a1: a1, zi0 };
        let mut result = OnePoleLowPassFilter {
            taps: ba.clone(),
            memory: ba,
        };
        result.reset();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{OnePoleError, OnePoleLowPassFilterBuilder};

    #[test]
    fn test_one() {
        OnePoleLowPassFilterBuilder::new()
            .alpha(0.99 as f32)
            .pass(super::FilterType::LowPass)
            .build()
            .expect("works");
    }

    #[test]
    fn test_fails() {
        let err = OnePoleLowPassFilterBuilder::new()
            .alpha(2.0 as f32)
            .pass(super::FilterType::HighPass)
            .build()
            .err()
            .unwrap_or_else(|| panic!("expecting an error"));
        assert!(matches!(err, OnePoleError::AlphaOutOfRange));
    }
}
