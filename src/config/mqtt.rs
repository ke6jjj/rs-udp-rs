use serde::Deserialize;

#[derive(Deserialize)]
pub struct MQTTConfig {
    /// Hostname or IP address of broker to contact.
    pub host: String,

    /// TCP port for MQTT connection.
    #[serde(default = "default_mqtt_port")]
    pub port: u16,

    /// MQTT client id (optional).
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,

    /// MQTT username (requires password, if set)
    pub username: Option<String>,

    /// MQTT password (requires username, if set)
    pub password: Option<String>,
}

fn default_mqtt_port() -> u16 {
    1883
}

fn default_mqtt_client_id() -> String {
    String::from("")
}
