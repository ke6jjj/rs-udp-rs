use std::path::PathBuf;

use crate::config::{FilterConfig, FlowConfig};
use crate::signal::{
    AffineError, AffineTransform, Event, EventBlock, EventGeneratingBlock, FilterObserver,
    FilterStep, LPFError, LowPassFilter, ObserverError, OnePoleError, OnePoleFilter,
    OnePoleFilterType, ProcessingBlock, Rectify, RectifyType, SignalBlock, ThresholdError,
    ThresholdTrigger,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowError {
    #[error("can't construct affine transform")]
    Affine(#[from] AffineError),
    #[error("can't construct one-pole dc filter")]
    DCOnePole(#[source] OnePoleError),
    #[error("can't construct one-pole ac filter")]
    ACOnePole(#[source] OnePoleError),
    #[error("can't construct filter")]
    FilterError(#[from] LPFError),
    #[error("can't set up trigger")]
    Trigger(#[source] ThresholdError),
    #[error("can't open debug dump file")]
    DebugDumpError(#[from] ObserverError),
}

pub struct TriggerResult {
    pub triggered: bool,
    pub reset: bool,
}

/// A reproduction of the all-in-one trigger processing flow that existed
/// before the signal block refactoring. This interface will disappear
/// and be replaced with one where the user needs to build their own
/// blocks in the configuration file.
pub struct ClassicTrigger {
    affine: ProcessingBlock<f32>,
    lpf: ProcessingBlock<f32>,
    dc_remove: ProcessingBlock<f32>,
    square: ProcessingBlock<f32>,
    ac_remove: ProcessingBlock<f32>,
    threshold: EventGeneratingBlock<f32>,
    processed: usize,
}

impl ClassicTrigger {
    pub fn process(
        &mut self,
        input: &ndarray::Array1<f32>,
        obs: &mut FilterObserver<f32>,
    ) -> TriggerResult {
        let n = self.processed;
        obs.observe(FilterStep::Input, n, input);
        let post_affine = self.affine.process(input);
        obs.observe(FilterStep::Affined, n, &post_affine);
        let post_lpf = self.lpf.process(&post_affine);
        obs.observe(FilterStep::Filtered, n, &post_lpf);
        let post_dc_remove = self.dc_remove.process(&post_lpf);
        obs.observe(FilterStep::DCRemove, n, &post_dc_remove);
        let post_square = self.square.process(&post_dc_remove);
        let post_ac_remove = self.ac_remove.process(&post_square);
        obs.observe(FilterStep::Energy, n, &post_ac_remove);
        let mut triggered = false;
        let mut reset = false;
        let obs = |event: Event<f32>| {
            match event {
                Event::Triggered(_when) => triggered = true,
                Event::Reset(_when) => reset = true,
                _ => (),
            };
        };
        self.threshold.process(&post_ac_remove, obs);
        self.processed += input.len();
        TriggerResult { triggered, reset }
    }
}

pub struct SensorFlow {
    pub trigger: ClassicTrigger,
    pub dumper: FilterObserver<f32>,
}

impl SensorFlow {
    pub fn new(trigger: ClassicTrigger, dumper: FilterObserver<f32>) -> Self {
        SensorFlow { dumper, trigger }
    }

    pub async fn from_config(
        sample_rate_hz: f32,
        flow_config: &FlowConfig,
        dump_override: Option<&PathBuf>,
    ) -> Result<SensorFlow, FlowError> {
        let trigger = trigger_from_config(sample_rate_hz, &flow_config.filter)?;
        let dump = match dump_override {
            Some(path) => FilterObserver::new_channel_dumper(path)?,
            None => FilterObserver::null()?,
        };
        Ok(SensorFlow::new(trigger, dump))
    }
}

fn trigger_from_config(
    sample_rate_hz: f32,
    filter: &FilterConfig,
) -> Result<ClassicTrigger, FlowError> {
    let affine: ProcessingBlock<f32> = AffineTransform::builder()
        .gain(filter.gain)
        .offset(filter.offset)
        .build()?
        .into();
    let lpf: ProcessingBlock<f32> = LowPassFilter::builder()
        .sample_rate(sample_rate_hz)
        .cutoff_hz(filter.cutoff)
        .order(filter.order as usize)
        .build()?
        .into();
    let dc_remove: ProcessingBlock<f32> = OnePoleFilter::builder()
        .alpha(filter.dc_alpha)
        .pass(OnePoleFilterType::HighPass)
        .build()
        .map_err(FlowError::DCOnePole)?
        .into();
    let square: ProcessingBlock<f32> = Rectify::builder()
        .rectify(RectifyType::Square)
        .build()
        .expect("how did you screw this one up?")
        .into();
    let ac_remove: ProcessingBlock<f32> = OnePoleFilter::builder()
        .alpha(filter.energy_alpha)
        .pass(OnePoleFilterType::LowPass)
        .build()
        .map_err(FlowError::ACOnePole)?
        .into();
    let threshold: EventGeneratingBlock<f32> = ThresholdTrigger::builder()
        .trigger(filter.trigger_level)
        .reset(filter.reset_level)
        .holdoff(filter.holdoff)
        .build()
        .map_err(FlowError::Trigger)?
        .into();
    let processed: usize = 0;
    let res = ClassicTrigger {
        affine,
        lpf,
        dc_remove,
        square,
        ac_remove,
        threshold,
        processed,
    };
    Ok(res)
}
