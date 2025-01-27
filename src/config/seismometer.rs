use super::flow::FlowConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SeismometerConfig {
    /// A name for the sensor
    pub name: String,

    /// The listen address ("ip:port") to listen on.
    pub listen: String,

    /// The sample rate of the seismometer, in hertz.
    /// Default: 100
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f32,

    /// How long to wait for data before declaring a timeout, in seconds.
    /// If provided, the timeout will be used to announce the "availability"
    /// of all flows from the seismometer. If not provided, no timeout will be used and the
    /// sensor will become "available" as soon as the program starts.
    pub timeout_s: Option<f32>,

    /// Filter and threshold settings.
    pub flows: Vec<FlowConfig>,
}

fn default_sample_rate() -> f32 {
    100.0
}
