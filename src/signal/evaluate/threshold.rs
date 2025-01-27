use std::iter::Sum;

use super::super::{Event, EventBlock};
use ndarray::ScalarOperand;
use num_traits::One;
use thiserror::Error;

pub use num_traits::{Float, Zero};
pub use sci_rs::na::RealField;

#[derive(Error, Debug)]
pub enum ThresholdError {
    #[error("trigger threshold is lower than reset threshold")]
    ThresholdError,
}

/// Signal processing block that judges whether a signal has gone above
/// or below a threshold level.
///
pub struct ThresholdTrigger<T>
where
    T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand,
{
    trigger: T,
    reset: T,
    triggered: bool,
    holdoff: usize,

    /// Number of samples processed so far.
    processed: usize,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> EventBlock<T>
    for ThresholdTrigger<T>
{
    fn reset(&mut self) {
        self.triggered = false;
        self.processed = 0;
    }

    fn process(&mut self, input: &ndarray::Array1<T>, mut obs: impl FnMut(Event<T>) -> ()) {
        for &v in input {
            if self.processed > self.holdoff {
                if !self.triggered && v > self.trigger {
                    obs(Event::Triggered(self.processed));
                    self.triggered = true
                }
                if self.triggered && v <= self.reset {
                    obs(Event::Reset(self.processed));
                    self.triggered = false
                }
            }
            self.processed += 1
        }
    }
}

pub struct ThresholdTriggerBuilder<T> {
    trigger: Option<T>,
    reset: Option<T>,
    holdoff: Option<usize>,
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> Default
    for ThresholdTriggerBuilder<T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RealField + Float + Copy + Sum + One + Zero + ScalarOperand> ThresholdTriggerBuilder<T> {
    pub fn new() -> Self {
        Self {
            trigger: None,
            reset: None,
            holdoff: None,
        }
    }

    /// Level at which to trigger.
    pub fn trigger(mut self, level: T) -> Self {
        self.trigger.replace(level);
        self
    }

    /// Level at which to reset trigger.
    pub fn reset(mut self, level: T) -> Self {
        self.reset.replace(level);
        self
    }

    /// Disable trigger until some number of samples have been processed.
    pub fn holdoff(mut self, n: usize) -> Self {
        self.holdoff.replace(n);
        self
    }

    /// Construct a trigger.
    pub fn build(self) -> Result<ThresholdTrigger<T>, ThresholdError> {
        let trigger = self.trigger.unwrap_or(T::one());
        let reset = self.reset.unwrap_or(T::zero());
        if trigger < reset {
            return Err(ThresholdError::ThresholdError);
        }
        let result = ThresholdTrigger {
            trigger,
            reset,
            triggered: false,
            holdoff: self.holdoff.unwrap_or(0),
            processed: 0,
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::ThresholdTriggerBuilder;

    #[test]
    fn test_one() {
        ThresholdTriggerBuilder::new()
            .trigger(0.5 as f32)
            .reset(0.2)
            .build()
            .expect("works");
    }
}
