pub use super::channel::Channel;
use super::data::SeismoData;
use super::rsudp::RSUDPFrame;
use core::str;
use std::io;
use thiserror::Error;
use tokio::net::UdpSocket;

use super::rsudp::RSUDPError;
#[derive(Error, Debug)]
pub enum UDPSourceError {
    #[error("error while attempting UDP bind")]
    UDPBindError(#[source] io::Error),
    #[error("UDP receive error")]
    UDPReceiveError(#[source] io::Error),
    #[error("unable to parse packet as utf-8")]
    UnparseableUTF8,
    #[error("packet decode error")]
    DecodeError(#[source] RSUDPError),
}

pub struct RSUDPSource {
    s: UdpSocket,
    channels: Option<Vec<bool>>,
    buf: [u8; 8192],
}

impl RSUDPSource {
    pub async fn new(listen_address: &str) -> Result<RSUDPSource, UDPSourceError> {
        let s = UdpSocket::bind(listen_address)
            .await
            .map_err(UDPSourceError::UDPBindError)?;
        Ok(RSUDPSource {
            s,
            channels: None,
            buf: [0_u8; 8192],
        })
    }

    pub fn subscribe(&mut self, channel: Channel) {
        let channel_interest = match self.channels.as_mut() {
            Some(existing_list) => existing_list,
            None => {
                let mut all_channels: Vec<bool> = Vec::with_capacity(Channel::max());
                for _channel in 0..Channel::max() {
                    all_channels.push(false)
                }
                self.channels = Some(all_channels);
                self.channels.as_mut().unwrap()
            }
        };
        channel_interest[channel as usize] = true;
    }

    async fn recv_packet(&mut self) -> Result<SeismoData, UDPSourceError> {
        loop {
            let packet_sz = self
                .s
                .recv(&mut self.buf)
                .await
                .map_err(UDPSourceError::UDPReceiveError)?;
            let buf = &self.buf[0..packet_sz];
            if let Some(data) = self.parse_packet(buf)? {
                return Ok(data);
            }
        }
    }

    pub fn parse_packet(&self, buf: &[u8]) -> Result<Option<SeismoData>, UDPSourceError> {
        let packet = str::from_utf8(buf).map_err(|_| UDPSourceError::UnparseableUTF8)?;
        let peek = RSUDPFrame::from_str(packet).map_err(UDPSourceError::DecodeError)?;
        if let Some(interested) = self.channels.as_ref() {
            if !interested[peek.channel as usize] {
                return Ok(None);
            }
        }
        let data = peek.decode().map_err(UDPSourceError::DecodeError)?;
        let result = SeismoData {
            timestamp: peek.timestamp,
            channel: peek.channel,
            data,
        };
        Ok(Some(result))
    }

    pub async fn next(&mut self) -> Option<Result<SeismoData, UDPSourceError>> {
        loop {
            match self.recv_packet().await {
                Ok(result) => return Some(Ok(result)),
                Err(e) => match e {
                    UDPSourceError::DecodeError(_) => continue,
                    UDPSourceError::UnparseableUTF8 => continue,
                    x => return Some(Err(x)),
                },
            }
        }
    }
}
