use super::action_loop::{ActionLoop, ActionLoopError};
use super::instrument_loop::{InstrumentLoop, LoopError};

use rumqttc::{ConnectionError, EventLoop};
use thiserror::Error;
use tokio::task::{JoinError, JoinSet};

#[derive(Debug, Error)]
pub enum AlarmSessionError {
    #[error("seismometer returned a data loop error")]
    DataLoop(#[from] LoopError),
    #[error("error waiting for seismometer loop")]
    LoopJoin(#[from] JoinError),
    #[error("MQTT connection failed")]
    MQTTConnection(#[from] ConnectionError),
    #[error("failure while taking action")]
    Action(#[from] ActionLoopError),
}

pub struct AlarmSession<'a> {
    /// A list of event loops that handle traffic from seismometers.
    instrument_loops: Vec<InstrumentLoop>,

    /// An event loop which will listen for events and take actions (publish
    /// to MQTT, run scripts).
    action_loop: ActionLoop<'a>,

    /// An optional MQTT event loop that must be run in order to provide
    /// MQTT service.
    mqtt_loop: Option<EventLoop>,
}

impl<'a> AlarmSession<'a> {
    pub fn new(
        instrument_loops: Vec<InstrumentLoop>,
        action_loop: ActionLoop<'a>,
        mqtt_loop: Option<EventLoop>,
    ) -> Self {
        Self {
            instrument_loops,
            action_loop,
            mqtt_loop,
        }
    }

    pub async fn run(self) -> Result<(), AlarmSessionError> {
        tokio::try_join!(
            Self::run_all_instrument_loops(self.instrument_loops),
            Self::run_mqtt_connection(self.mqtt_loop),
            Self::run_actions_loop(self.action_loop),
        )?;
        Ok(())
    }

    async fn run_all_instrument_loops(
        sensors: Vec<InstrumentLoop>,
    ) -> Result<(), AlarmSessionError> {
        let mut monitor_tasks = JoinSet::new();
        for instrument in sensors {
            monitor_tasks.spawn(instrument.run());
        }

        while let Some(res) = monitor_tasks.join_next().await {
            res??
        }
        Ok(())
    }

    async fn run_mqtt_connection(
        mqtt_event_loop: Option<EventLoop>,
    ) -> Result<(), AlarmSessionError> {
        if let Some(mut conn) = mqtt_event_loop {
            loop {
                let _event = conn.poll().await?;
            }
        }
        Ok(())
    }

    async fn run_actions_loop(
        action_loop: ActionLoop<'a>
    ) -> Result<(), AlarmSessionError> {
        action_loop.run().await?;
        Ok(())
    }
}
