use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilterConfig {
    /// Energy level required to enable the trigger (after all filtering)
    #[serde(default = "default_trigger_level")]
    pub trigger_level: f32,

    /// Energy level requried to reset the trigger
    #[serde(default = "default_reset_level")]
    pub reset_level: f32,

    /// A value to remove from every sample before processing.
    #[serde(default = "default_offset")]
    pub offset: f32,

    /// A value to mutiply each sample by after removing any offset.
    #[serde(default = "default_gain")]
    pub gain: f32,

    /// The order of the low pass filter to create.
    /// Default: 8
    #[serde(default = "default_filter_order")]
    pub order: u8,

    /// The cutoff frequency for the detection filter, in hertz.
    /// Default: 8.
    #[serde(default = "default_cutoff_freq")]
    pub cutoff: f32,

    /// DC-offset tracking decay rate/'alpha'
    /// Default: .99
    #[serde(default = "default_dc_alpha")]
    pub dc_alpha: f32,

    /// Energy detection decay rate/'alpha'
    /// Default: .99
    #[serde(default = "default_energy_alpha")]
    pub energy_alpha: f32,

    /// Number of samples to process before enabling trigger.
    #[serde(default = "default_holdoff")]
    pub holdoff: usize,
}

fn default_trigger_level() -> f32 {
    1.0
}

fn default_reset_level() -> f32 {
    0.0
}

fn default_offset() -> f32 {
    0.0
}

fn default_gain() -> f32 {
    1.0
}

fn default_filter_order() -> u8 {
    8
}

fn default_cutoff_freq() -> f32 {
    8.0
}

fn default_dc_alpha() -> f32 {
    0.99
}

fn default_energy_alpha() -> f32 {
    0.99
}

fn default_holdoff() -> usize {
    0
}
