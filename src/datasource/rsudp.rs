use super::channel::{Channel, ChannelError};
use ndarray::{self, Array1};
use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RSUDPError {
    #[error("packet too small")]
    PacketTooSmall,
    #[error("packet not delimited with curly braces")]
    NotCurlyDelimited,
    #[error("no parts found in packet")]
    NoPartsFound,
    #[error("channel word not quoted")]
    ChannelWordNotQuoted,
    #[error("empty channel name")]
    EmptyChannelName,
    #[error("nothing after channel name")]
    NothingAfterChannelName,
    #[error("unsupported channel")]
    UnsupportedChannelName(#[from] ChannelError),
    #[error("unparsable timestamp")]
    UnparsableTimestamp,
    #[error("unparsable data")]
    UnparsableData,
}

// An intermediate result from inspecting a packet. Mostly used for
// strategically skipping parsing of uninteresting channels.
pub struct RSUDPFrame<'a> {
    pub channel: Channel,
    pub timestamp: f64,
    pub data: &'a str,
}

impl<'a> RSUDPFrame<'a> {
    pub fn from_str(s: &'a str) -> Result<RSUDPFrame<'a>, RSUDPError> {
        if s.len() <= 2 {
            return Err(RSUDPError::PacketTooSmall);
        }
        if !s.starts_with("{") || !s.ends_with("}") {
            return Err(RSUDPError::NotCurlyDelimited);
        }
        let body = s.get(1..s.len()).expect("impossible");
        let (channel_word_r, rest) = body.split_once(",").ok_or(RSUDPError::NoPartsFound)?;
        let channel_word = channel_word_r.trim();
        if !channel_word.starts_with("'") || !channel_word.ends_with("'") {
            return Err(RSUDPError::ChannelWordNotQuoted);
        }
        if channel_word.len() <= 2 {
            return Err(RSUDPError::EmptyChannelName);
        }
        let channel_name = channel_word
            .get(1..channel_word.len() - 1)
            .expect("impossble");
        let channel: Channel = channel_name.try_into()?;
        let (timestamp_s, the_rest) = rest
            .split_once(",")
            .ok_or(RSUDPError::NothingAfterChannelName)?;
        let timestamp: f64 = timestamp_s
            .trim()
            .parse()
            .ok()
            .ok_or(RSUDPError::UnparsableTimestamp)?;
        let result = RSUDPFrame {
            channel,
            timestamp,
            data: &the_rest[0..the_rest.len() - 1],
        };
        Ok(result)
    }

    pub fn decode(&self) -> Result<Array1<f32>, RSUDPError> {
        let data = self
            .data
            .split(",")
            .map(|s| s.trim().parse::<f32>())
            .collect::<Result<Vec<f32>, ParseFloatError>>()
            .ok()
            .ok_or(RSUDPError::UnparsableData)?;
        let array = ndarray::Array1::from_iter(data);
        Ok(array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let peeked = RSUDPFrame::from_str("{'EHZ',12345678.000,0,1,2,3,4,5,6}");
        let stuff = peeked.unwrap();
        assert_eq!(stuff.channel, Channel::Ehz);
        stuff.decode().unwrap();
    }

    #[test]
    fn it_errors() {
        let peeked = RSUDPFrame::from_str("{'EHZ,12345678.000,0,1,2,3,4,5,6}");
        assert!(peeked.is_err())
    }

    #[test]
    fn it_errors_empty() {
        let peeked = RSUDPFrame::from_str("{'',12345678.000,0,1,2,3,4,5,6}");
        assert!(peeked.is_err())
    }

    #[test]
    fn it_errors_garbage() {
        let peeked = RSUDPFrame::from_str("{'EHZ',12345678.000,0,1,2,3,Q,5,6}");
        let decoded = peeked.unwrap().decode();
        assert!(decoded.is_err())
    }

    #[test]
    fn it_errors_garbage_timestamp() {
        let peeked = RSUDPFrame::from_str("{'EHZ',garbage,0,1,2,3,Q,5,6}");
        assert!(peeked.is_err())
    }

    #[test]
    fn spaces_ok() {
        let peeked = RSUDPFrame::from_str("{'EHZ',12345678.000,0,1, 2,3, 4,5,6}");
        peeked.unwrap().decode().unwrap();
    }

    #[test]
    fn real_example() {
        let peeked = RSUDPFrame::from_str("{'EHZ', 1734044506.042, 16603, 16729, 16864, 16951, 16524, 15927, 15714, 15902, 16285, 16659, 16835, 16801, 16792, 16665, 16431, 16001, 15886, 16063, 16195, 16699, 17041, 16923, 16739, 16392, 16040}").unwrap();
        assert_eq!(peeked.timestamp, 1734044506.042);
        let data = peeked.decode().unwrap();
        assert_eq!(data[0], 16603.0);
    }
}
