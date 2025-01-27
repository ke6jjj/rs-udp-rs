mod channel;
mod data;
mod rsudp;
mod txtfile;
mod udp_source;

pub use channel::Channel;
pub use channel::ChannelError;
pub use data::SeismoData;

use std::path::Path;
use thiserror::Error;
use txtfile::{TextFileSource, TextSourceError};
use udp_source::{RSUDPSource, UDPSourceError};

#[derive(Error, Debug)]
pub enum DataSourceError {
    #[error("usp source error")]
    UDPSourceError(#[from] UDPSourceError),
    #[error("text parse error")]
    TextSourceError(#[from] TextSourceError),
}
pub enum DataSource {
    UDPSource(RSUDPSource),
    TextSource(TextFileSource),
}

impl DataSource {
    pub async fn new_rsudp_source(listen_address: &str) -> Result<DataSource, DataSourceError> {
        let ds = RSUDPSource::new(listen_address).await?;
        Ok(DataSource::UDPSource(ds))
    }

    pub async fn new_textfile_source(
        path: &Path,
        as_channel: Channel,
    ) -> Result<DataSource, DataSourceError> {
        let ds = TextFileSource::new(path, as_channel).await?;
        Ok(DataSource::TextSource(ds))
    }

    pub fn subscribe(&mut self, channel: Channel) {
        match self {
            DataSource::UDPSource(s) => s.subscribe(channel),
            DataSource::TextSource(s) => s.subscribe(channel),
        }
    }

    pub async fn next(&mut self) -> Option<Result<SeismoData, DataSourceError>> {
        match self {
            DataSource::UDPSource(s) => s
                .next()
                .await
                .map(|i| i.map_err(DataSourceError::UDPSourceError)),
            DataSource::TextSource(s) => s
                .next()
                .await
                .map(|i| i.map_err(DataSourceError::TextSourceError)),
        }
    }
}
