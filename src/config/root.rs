use super::mqtt::MQTTConfig;
use super::seismometer::SeismometerConfig;

use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("configuration error")]
    ParseError(#[from] ConfigError),
}

#[derive(Deserialize)]
pub struct Config {
    /// A list of seismometers to monitor.
    pub seismometers: Vec<SeismometerConfig>,

    /// MQTT settings.
    pub mqtt: Option<MQTTConfig>,
}

impl Config {
    pub fn new(
        path: &Path,
        env_prefix: &str,
        env_separator: &str,
    ) -> Result<Self, ConfigurationError> {
        let config_file =
            File::with_name(path.to_str().expect("file name")).format(FileFormat::Json);
        config::Config::builder()
            .add_source(config_file)
            .add_source(Environment::with_prefix(env_prefix).separator(env_separator))
            .build()
            .and_then(|config| config.try_deserialize())
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_decodes() {
        let _c: Config = serde_json::from_str("{\"seismometers\": []}").expect("parse");
    }
}
