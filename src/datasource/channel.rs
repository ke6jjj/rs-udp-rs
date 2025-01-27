use thiserror::Error;
use variant_count::VariantCount;

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("no such channel")]
    NoSuchChannel,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, VariantCount)]
pub enum Channel {
    Ehz,
    Ehn,
    Ehe,
    Enz,
    Enn,
    Ene,
}

impl Channel {
    pub const fn max() -> usize {
        Channel::VARIANT_COUNT
    }
}

impl From<Channel> for usize {
    fn from(value: Channel) -> Self {
        match value {
            Channel::Ehz => 0,
            Channel::Ehn => 1,
            Channel::Ehe => 2,
            Channel::Enz => 3,
            Channel::Enn => 4,
            Channel::Ene => 5,
        }
    }
}

impl TryFrom<usize> for Channel {
    type Error = ChannelError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Channel::Ehz,
            1 => Channel::Ehn,
            2 => Channel::Ehe,
            3 => Channel::Enz,
            4 => Channel::Enn,
            5 => Channel::Ene,
            _ => return Err(ChannelError::NoSuchChannel),
        };
        Ok(res)
    }
}

impl TryFrom<&str> for Channel {
    type Error = ChannelError;

    /// Only works for uppercase inputs.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let res = match value {
            "EHZ" => Self::Ehz,
            "EHN" => Self::Ehn,
            "EHE" => Self::Ehe,
            "ENZ" => Self::Enz,
            "ENN" => Self::Enn,
            "ENE" => Self::Ene,
            _ => return Err(ChannelError::NoSuchChannel),
        };
        Ok(res)
    }
}
