use crate::config::ActionsConfig;

use rumqttc::{AsyncClient, ClientError};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum ActionLoopError {
    #[error("error publishing MQTT topic")]
    MQTTClientError(#[from] ClientError),
    #[error("failed to execute external program")]
    ExecuteFailure(#[from] std::io::Error),
}

/// A seismometer event.
pub enum Event {
    Status { dc: f32, energy: f32 },
    Available,
    Unavailable,
    Triggered,
    Reset,
}

/// A seismometer event from a particular seismometer.
pub struct TriggerMessage {
    pub source_id: usize,
    pub event: Event,
}

struct Flow<'a> {
    name: &'a str,
    actions: &'a ActionsConfig,
}

/// A set of actions to take on seismometer events, indexed by siesmometer.
type FlowsMap<'a> = HashMap<usize, Flow<'a>>;

/// An asynchronous channel for sending seismometer events to the main thread.
pub type OutChannel = tokio::sync::mpsc::Sender<TriggerMessage>;

/// An asynchronous channel for receiving events from all seismometers.
pub type InChannel = tokio::sync::mpsc::Receiver<TriggerMessage>;

/// Construct a channel pair for seismometers to post events into, and
/// from which the main thread can receive them.
pub fn message_channel() -> (OutChannel, InChannel) {
    tokio::sync::mpsc::channel::<TriggerMessage>(32)
}

pub struct ActionLoop<'a> {
    flows: FlowsMap<'a>,
    mqtt: Option<AsyncClient>,
    chan: InChannel,
}

impl<'a> ActionLoop<'a> {
    pub fn new(chan: InChannel, mqtt: Option<AsyncClient>) -> Self {
        Self {
            flows: FlowsMap::new(),
            chan,
            mqtt,
         }
    }

    /// Introduce a new sensor and its actions to the loop.
    pub fn add_flow(&mut self, flow_id: usize, name: &'a str, actions: &'a ActionsConfig) {
        let flow = Flow { name, actions };
        self.flows.insert(flow_id, flow);
    }

    /// Listen for events from all seismometers. When they are received, take
    /// action on them from the configured actions.
    pub async fn run(mut self) -> Result<(), ActionLoopError> {
        while let Some(msg) = self.chan.recv().await {
            self.handle_seismometer_event(msg).await?;
        }
        Ok(())
    }

    /// Handle an event that has been noted by a particular seismometer.
    async fn handle_seismometer_event(&mut self, msg: TriggerMessage) -> Result<(), ActionLoopError> {
        //
        // Look up the reporting seismometer and see if there are any actions
        // configured for its events.
        //
        if let Some(flow) = self.flows.get(&msg.source_id) {
            let actions = flow.actions;
            let name = flow.name;
            match msg.event {
                //
                // A seismometer appears to have come online.
                //
                Event::Available => {
                    tokio::try_join!(
                        self.mqtt_publish(
                            &actions.mqtt_available_topic,
                            &actions.mqtt_available_payload,
                        ),
                        cmd_run(&actions.available_cmd, "available", name)
                    )?;
                }

                //
                // A seismometer is reporting a running status.
                //
                Event::Status {
                    dc: _dc,
                    energy: _energy,
                } => {}

                //
                // A seismometer is reporting an earthquake.
                //
                Event::Triggered => {
                    tokio::try_join!(
                        self.mqtt_publish(
                            &actions.mqtt_topic,
                            &actions.mqtt_triggered_payload,
                        ),
                        cmd_run(&actions.trigger_cmd, "triggered", name)
                    )?;
                }

                //
                // A seismometer that was previously reporting an earthquake
                // is now no longer reporting one.
                //
                Event::Reset => {
                    tokio::try_join!(
                        self.mqtt_publish(
                            &actions.mqtt_topic,
                            &actions.mqtt_reset_payload,
                        ),
                        cmd_run(&actions.reset_cmd, "reset", name)
                    )?;
                }

                //
                // A seismometer is reporting that it has come online.
                //
                Event::Unavailable => {
                    tokio::try_join!(
                        self.mqtt_publish(
                            &actions.mqtt_available_topic,
                            &actions.mqtt_unavailable_payload,
                        ),
                        cmd_run(&actions.unavailable_cmd, "unavailable", name)
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Publish a payload over MQTT, but only if so configured.
    async fn mqtt_publish(&mut self, topic: &Option<String>, payload: &String) -> Result<(), ActionLoopError> {
        let config = self.mqtt.as_mut().zip(topic.as_ref());
        if let Some((client, topic)) = config {
            client
                .publish(
                    topic.as_str(),
                    rumqttc::QoS::AtLeastOnce,
                    false,
                    payload.as_bytes(),
                )
                .await?;
        }
        Ok(())
    }

}

/// Execute an external executable, if so configured.
async fn cmd_run(cmd: &Option<PathBuf>, arg1: &str, arg2: &str) -> Result<(), ActionLoopError> {
    if let Some(path) = cmd.as_ref() {
        let _ = Command::new(path)
            .args([ arg1, arg2 ])
            .status().await?;
    }
    Ok(())
}
