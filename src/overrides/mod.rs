use crate::datasource::{Channel, ChannelError};
use std::{path::PathBuf, str::FromStr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SeismometerOverrideError {
    #[error("override spec missing channel=input separator")]
    MissingPathSeparator,
    #[error("override spec missing sensor:channel separator")]
    MissingChannelSeparator,
    #[error("unknown seismometer channel")]
    UnknownChannel(#[from] ChannelError),
}

#[derive(Error, Debug)]
pub enum FlowDumpError {
    #[error("flow dump spec channel=input separator")]
    MissingPathSeparator,
    #[error("override spec missing sensor:channel separator")]
    MissingChannelSeparator,
}

#[derive(Debug, Clone)]
/// A specification that pairs a text file with a seismometer so as
/// to completely replace that seismometer with a datastream coming
/// from the text file, masquerading as data for a specific channel.
pub struct SeismometerTiedPath {
    pub seismometer_name: String,
    pub channel: Channel,
    pub path: PathBuf,
}

impl FromStr for SeismometerTiedPath {
    type Err = SeismometerOverrideError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (sensor_name, channel, path) = s
            .split_once('=')
            .ok_or(SeismometerOverrideError::MissingPathSeparator)
            .and_then(|(sensor, after)| {
                after
                    .split_once(':')
                    .ok_or(SeismometerOverrideError::MissingChannelSeparator)
                    .map(|(channel, path)| (sensor, channel, path))
            })?;
        Ok(Self {
            seismometer_name: sensor_name.to_owned(),
            channel: channel.try_into()?,
            path: path.into(),
        })
    }
}

#[derive(Debug, Clone)]
/// A specification that pairs a text file with a signal flow's output,
/// typically to ask that a copy of a diagnostic data stream from the flow be
/// written to a file.
pub struct FlowTiedPath {
    pub flow_name: String,
    pub path: PathBuf,
}

impl FromStr for FlowTiedPath {
    type Err = FlowDumpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (flow_name, path) = s
            .split_once('=')
            .ok_or(FlowDumpError::MissingPathSeparator)?;
        Ok(Self {
            flow_name: flow_name.to_owned(),
            path: path.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::SeismometerTiedPath;

    #[test]
    fn test_one() {
        SeismometerTiedPath::from_str("shake4d=EHZ:/tmp/test").expect("works");
    }
}
