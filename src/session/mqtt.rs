use crate::config::Config;
use rumqttc::{AsyncClient, EventLoop, MqttOptions};

pub struct MQTT(pub Option<AsyncClient>, pub Option<EventLoop>);

impl MQTT {
    pub fn from_config(config: &Config) -> MQTT {
        let mqtt_config = match config.mqtt.as_ref() {
            None => return MQTT(None, None),
            Some(mqtt_config) => mqtt_config,
        };
        let mut options =
            MqttOptions::new(&mqtt_config.client_id, &mqtt_config.host, mqtt_config.port);
        mqtt_config
            .username
            .as_ref()
            .zip(mqtt_config.password.as_ref())
            .and_then(|(username, password)| {
                options.set_credentials(username, password);
                None::<()>
            });
        let (client, event_loop) = AsyncClient::new(options, 10);
        MQTT(Some(client), Some(event_loop))
    }
}
