use super::channel::Channel;
use ndarray::Array1;

pub struct SeismoData {
    #[allow(dead_code)]
    pub timestamp: f64,
    pub channel: Channel,
    pub data: Array1<f32>,
}
