use tokio::task::JoinError;
use tokio::time::{Duration, Instant};

use super::action_loop::{Event, OutChannel, TriggerMessage};
use super::sensor_flow::SensorFlow;
use super::timeout::ChannelChecker;
use crate::datasource::{Channel, DataSource, DataSourceError, SeismoData};

use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Error, Debug)]
pub enum LoopError {
    #[error("Message send failure")]
    SendFailure(#[from] SendError<TriggerMessage>),
    #[error("Data source error")]
    DataSourceError(#[from] DataSourceError),
    #[error("Error joining async spawn")]
    JoinError(#[from] JoinError),
}

struct FlowState {
    flow_id: usize,
    flow: SensorFlow,
    triggered: Option<bool>,
}

pub struct InstrumentLoop {
    src: DataSource,
    flows_for_channel: Vec<Vec<FlowState>>,
    action_channel: OutChannel,
    timeouts_by_channel: ChannelChecker,
}

impl InstrumentLoop {
    // Construct a new instrument loop that pulls data from the given
    // data source, passes it through various signal flows, and signals
    // various events based on the results.
    pub fn new_for_datasource(
        src: DataSource,
        timeout_s: Option<f32>,
        action_channel: OutChannel,
    ) -> InstrumentLoop {
        let timeout = timeout_s.map(Duration::from_secs_f32);
        let mut flows_for_channel = Vec::with_capacity(Channel::max());
        flows_for_channel.extend((0..Channel::max()).map(|_| Vec::new()));

        InstrumentLoop {
            flows_for_channel,
            src,
            action_channel,
            timeouts_by_channel: ChannelChecker::new_for_timeout(timeout),
        }
    }

    pub fn add_flow(&mut self, flow_id: usize, channel: Channel, flow: SensorFlow) {
        let state = FlowState {
            flow_id,
            flow,
            triggered: None,
        };
        self.timeouts_by_channel.track_channel(channel);
        self.flows_for_channel[channel as usize].push(state);
        self.src.subscribe(channel);
    }

    pub async fn run(mut self) -> Result<(), LoopError> {
        self.timeouts_by_channel.start(Instant::now());

        loop {
            tokio::select! {
                frame = self.src.next() => {
                    match frame {
                        Some(data_result) => self.handle_data(data_result?, Instant::now()).await?,
                        None => break,
                    };
                },
                _ = tokio::time::sleep(self.timeouts_by_channel.next_timeout(Instant::now()).unwrap_or(Duration::MAX)) => {
                    // One or more channels just timed out
                    self.handle_timeout(Instant::now()).await?;
                },
            }
        }
        Ok(())
    }

    async fn handle_timeout(&mut self, when: Instant) -> Result<(), LoopError> {
        for channel_state in self.timeouts_by_channel.timeout_iter(when) {
            for flow in self.flows_for_channel[channel_state.channel as usize].iter() {
                flow.unavailable(&self.action_channel).await?;
            }
        }
        Ok(())
    }

    async fn handle_data(&mut self, data: SeismoData, when: Instant) -> Result<(), LoopError> {
        //
        // We have a valid new frame. If the source was previously
        // marked "offline", or it hasn't ever been seen yet,
        // mark it "online".
        //
        let already_active = self
            .timeouts_by_channel
            .mark_channel_alive(when, data.channel);
        for flow in self.flows_for_channel[data.channel as usize].iter_mut() {
            if ! already_active {
                flow.available(&self.action_channel).await?;
                flow.reset(&self.action_channel).await?;
            }
            flow.process(&data, &self.action_channel).await?;
        }
        Ok(())
    }
}

impl FlowState {
    pub async fn process(
        &mut self,
        input: &SeismoData,
        post: &OutChannel,
    ) -> Result<(), LoopError> {
        let result = self
            .flow.trigger.process(&input.data, &mut self.flow.dumper);
        if result.triggered {
            self.triggered(post).await?;
        }
        if result.reset {
            self.reset(post).await?;
        }
        Ok(())
    }

    pub async fn available(&self, channel: &OutChannel) -> Result<(), LoopError> {
        self.send_event(Event::Available, channel).await?;
        Ok(())
    }

    pub async fn unavailable(&self, channel: &OutChannel) -> Result<(), LoopError> {
        self.send_event(Event::Unavailable, channel).await?;
        Ok(())
    }

    pub async fn triggered(&mut self, channel: &OutChannel) -> Result<(), LoopError> {
        if !self.triggered.unwrap_or(false) {
            self.send_event(Event::Triggered, channel).await?;
            self.triggered.replace(true);
        }
        Ok(())
    }

    pub async fn reset(&mut self, channel: &OutChannel) -> Result<(), LoopError> {
        if self.triggered.unwrap_or(true) {
            self.send_event(Event::Reset, channel).await?;
            self.triggered.replace(false);
        }
        Ok(())
    }

    pub async fn send_event(&self, event: Event, channel: &OutChannel) -> Result<(), LoopError> {
        channel
            .send(TriggerMessage {
                source_id: self.flow_id,
                event,
            })
            .await?;
        Ok(())
    }
}
