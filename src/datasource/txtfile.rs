use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};
use thiserror::Error;

pub use super::channel::Channel;
use super::data::SeismoData;

#[derive(Error, Debug)]
pub enum TextSourceError {
    #[error("unable to open file")]
    FileOpenFailed(#[source] std::io::Error),
    #[error("bad data split, got {0} parts instead of 2")]
    BadDataSplit(usize),
    #[error("bad line read")]
    BadLineRead(#[source] std::io::Error),
    #[error("unparseable float")]
    UnparsableFloat,
}

pub struct TextFileSource {
    channel: Channel,
    f: Option<File>,
}

fn handle_line(line: &str) -> Result<f32, TextSourceError> {
    let parts: Vec<&str> = line.split_ascii_whitespace().collect();
    if parts.len() < 2 {
        return Err(TextSourceError::BadDataSplit(parts.len()));
    }
    let v = parts[1]
        .parse::<f32>()
        .map_err(|_| TextSourceError::UnparsableFloat)?;
    Ok(v)
}

fn read_file(f: File, as_channel: Channel) -> Result<SeismoData, TextSourceError> {
    let data = io::BufReader::new(f)
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()
        .map_err(TextSourceError::BadLineRead)?
        .into_iter()
        .map(|line| handle_line(&line))
        .collect::<Result<Vec<f32>, TextSourceError>>()?;
    let result = SeismoData {
        timestamp: 0.0,
        channel: as_channel,
        data: ndarray::Array1::from_iter(data),
    };
    Ok(result)
}

impl TextFileSource {
    pub async fn new(path: &Path, as_channel: Channel) -> Result<TextFileSource, TextSourceError> {
        let f = File::open(path).map_err(TextSourceError::FileOpenFailed)?;
        Ok(TextFileSource {
            channel: as_channel,
            f: Some(f),
        })
    }

    pub async fn next(&mut self) -> Option<Result<SeismoData, TextSourceError>> {
        self.f.take().map(|f| read_file(f, self.channel))
    }

    pub fn subscribe(&mut self, _: Channel) {}
}
