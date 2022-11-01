use cpal::{Device, SupportedStreamConfig};
use std::time::Duration;
use vosk::Model;

pub struct Config {
    pub model: Model,
    pub record_duration: Duration,
    pub supported_stream_config: SupportedStreamConfig,
    pub audio_input_device: Device,
    pub channels: u16,
}
