use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ActionsConfig {
    /// Executable to spawn when seismometer is deemed to be sending
    /// data and running.
    pub available_cmd: Option<PathBuf>,

    /// Executable to spawn when seismometer is deemed to be offline
    /// and not sending data.
    pub unavailable_cmd: Option<PathBuf>,

    /// Executable to spawn when the seismometer filter detects
    /// enough energy to trip its internal trigger. (When an earthquake
    /// is happening).
    pub trigger_cmd: Option<PathBuf>,

    /// Executable to spawn when the seismometer filter transitions from
    /// triggered state to calm state. (When an earthquake is over).
    pub reset_cmd: Option<PathBuf>,

    /// MQTT topic to post to when an earthquake is detected.
    pub mqtt_topic: Option<String>,

    /// MQTT topic to post to when a sensor is detected as online, or
    /// seemds to have timed out.
    pub mqtt_available_topic: Option<String>,

    /// Payload to post to main topic when an earthquake is detected.
    /// Will be sent in UTF-8 encoding.
    /// (Only used if mqtt_topic is present.)
    #[serde(default = "default_on_payload")]
    pub mqtt_triggered_payload: String,

    /// Payload to post to main topic when an earthquake has subsided.
    /// Will be sent in UTF-8 encoding.
    /// (Only used if mqtt_topic is present.)
    #[serde(default = "default_off_payload")]
    pub mqtt_reset_payload: String,

    /// Payload to post to availability topic when the sensor is detected
    /// as being online.
    /// Will be sent in UTF-8 encoding.
    /// (Only used if mqtt_availabile_topic is present.)
    #[serde(default = "default_on_payload")]
    pub mqtt_available_payload: String,

    /// Payload to post to availability topic when the sensor is detected
    /// as being offline.
    /// Will be sent in UTF-8 encoding.
    /// (Only used if mqtt_availabile_topic is present.)
    #[serde(default = "default_off_payload")]
    pub mqtt_unavailable_payload: String,
}

fn default_on_payload() -> String {
    String::from("ON")
}

fn default_off_payload() -> String {
    String::from("OFF")
}
