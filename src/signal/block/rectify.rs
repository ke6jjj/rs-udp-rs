use std::iter::Sum;

use ndarray::ScalarOperand;
use num_traits::One;
use thiserror::Error;

pub use num_traits::{Float, Zero};
pub use sci_rs::na::RealField;

use crate::signal::SignalBlock;

#[derive(Clone, Copy, Default)]
pub enum RectifyType {
    #[default]
    Absolute,
    Square,
}

#[derive(Error, Debug)]
pub enum RectifyError {}

/// Signal processor that accepts voltage measurements from a seismic
/// sensor and detects the likely presence of an earthquake according to
/// some criteria.
///
/// Signal flow (per sample)
///
/// 1. Subtract "offset" from sample.
/// 2. Multiply result by "gain".
/// 3. Send through low-pass filter.
/// 4. Send through moving-average DC removal filter.
/// 5. Square result (to obtain instantaneous power).
/// 6. Send through moving-average energy filter.
/// 7. Trigger/Reset on result.
///
pub struct Rectify {
    rectify_type: RectifyType,
}

impl Rectify {
    pub fn builder() -> RectifyBuilder {
        RectifyBuilder::new()
    }
}

impl<T> SignalBlock<T> for Rectify
where
    T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand,
{
    fn reset(&mut self) {}

    fn process(&mut self, input: &ndarray::Array1<T>) -> ndarray::Array1<T> {
        match self.rectify_type {
            RectifyType::Square => input.pow2(),
            RectifyType::Absolute => input.abs(),
        }
    }
}

#[derive(Default)]
pub struct RectifyBuilder {
    rectify_type: Option<RectifyType>,
}

impl RectifyBuilder {
    pub fn new() -> Self {
        Self { rectify_type: None }
    }
}

impl RectifyBuilder {
    /// Rectification type.
    pub fn rectify(mut self, rtype: RectifyType) -> Self {
        self.rectify_type.replace(rtype);
        self
    }

    /// Construct a trigger.
    pub fn build(self) -> Result<Rectify, RectifyError> {
        let rectify_type = self.rectify_type.unwrap_or_default();
        Ok(Rectify { rectify_type })
    }
}

#[cfg(test)]
mod tests {
    use super::{RectifyBuilder, RectifyType};

    #[test]
    fn test_one() {
        RectifyBuilder::new()
            .rectify(RectifyType::Absolute)
            .build()
            .expect("works");
    }
}
