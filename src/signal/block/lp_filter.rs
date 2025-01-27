use std::iter::Sum;

use ndarray::ScalarOperand;
use num_traits::One;
use sci_rs::signal::filter::design::butter_dyn;
use sci_rs::signal::filter::design::DigitalFilter;
use sci_rs::signal::filter::design::FilterBandType;
use sci_rs::signal::filter::design::FilterOutputType;
use sci_rs::signal::filter::design::Sos;
use sci_rs::signal::filter::design::SosFormatFilter;
use sci_rs::signal::filter::sosfilt_dyn;
use thiserror::Error;

pub use num_traits::{Float, Zero};
pub use sci_rs::na::RealField;

use crate::signal::SignalBlock;

#[derive(Error, Debug)]
pub enum LPFError {
    #[error("failed to create filter")]
    FilterFailure,
    #[error("cutoff frequency is too high for sample rate")]
    CutoffTooHigh,
}

/// Signal processor that accepts voltage measurements from a seismic
/// sensor and detects the likely presence of an earthquake according to
/// some criteria.
///
/// Signal flow (per sample)
///
/// 1. Send through low-pass filter.
pub struct LowPassFilter<T>
where
    T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand,
{
    taps: Vec<Sos<T>>,
    memory: Vec<Sos<T>>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> SignalBlock<T>
    for LowPassFilter<T>
{
    fn reset(&mut self) {
        self.memory = self.taps.clone();
    }

    fn process(&mut self, input: &ndarray::Array1<T>) -> ndarray::Array1<T> {
        ndarray::Array1::from_iter(sosfilt_dyn(input, self.memory.as_mut_slice()))
    }
}

pub struct LowPassFilterBuilder<T> {
    sample_rate_hz: Option<T>,
    cutoff_hz: Option<T>,
    order: Option<usize>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> Default
    for LowPassFilterBuilder<T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> LowPassFilterBuilder<T> {
    pub fn new() -> Self {
        Self {
            sample_rate_hz: None,
            cutoff_hz: None,
            order: None,
        }
    }

    /// Interpret samples as coming in at a sample rate.
    pub fn sample_rate(mut self, hz: T) -> Self {
        self.sample_rate_hz.replace(hz);
        self
    }

    /// Low-pass filter order.
    pub fn order(mut self, order: usize) -> Self {
        self.order.replace(order);
        self
    }

    /// Low-pass fitlter cutoff frequency.
    pub fn cutoff_hz(mut self, hz: T) -> Self {
        self.cutoff_hz.replace(hz);
        self
    }

    /// Construct a low-pass filter block.
    pub fn build(self) -> Result<LowPassFilter<T>, LPFError> {
        let cutoff_hz = self.cutoff_hz.unwrap_or(T::one());
        let sample_rate_hz = self.sample_rate_hz.unwrap_or(T::one() + T::one());
        if sample_rate_hz < cutoff_hz {
            return Err(LPFError::CutoffTooHigh);
        }
        let filter = butter_dyn(
            self.order.unwrap_or(4),
            [cutoff_hz].to_vec(),
            Some(FilterBandType::Lowpass),
            Some(false),
            Some(FilterOutputType::Sos),
            Some(sample_rate_hz),
        );
        let DigitalFilter::Sos(SosFormatFilter { sos }) = filter else {
            return Err(LPFError::FilterFailure);
        };
        let mut result = LowPassFilter {
            taps: sos,
            memory: [].to_vec(),
        };
        result.reset();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::LowPassFilterBuilder;

    #[test]
    fn test_one() {
        LowPassFilterBuilder::new()
            .sample_rate(100.0)
            .cutoff_hz(6.0)
            .order(4)
            .build()
            .expect("works");
    }
}
