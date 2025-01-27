use super::actions::ActionsConfig;
use super::filter::FilterConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FlowConfig {
    /// A name for the flow (so that it can be targetted later).
    pub name: String,

    /// The channel to observe from the seismometer.
    pub channel: String,

    /// Filter and trigger parameters.
    pub filter: FilterConfig,

    /// Actions to take on events.
    pub actions: ActionsConfig,
}
