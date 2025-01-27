use super::mqtt::MQTTConfig;
use super::seismometer::SeismometerConfig;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    /// A list of seismometers to monitor.
    pub seismometers: Vec<SeismometerConfig>,

    /// MQTT settings.
    pub mqtt: Option<MQTTConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes() {
        let _c: Config = serde_json::from_str("{\"seismometers\": []}").expect("parse");
    }
}
