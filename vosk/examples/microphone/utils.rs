use super::models::Config;
use cpal::traits::{DeviceTrait, HostTrait};
use std::{env, time::Duration};
use vosk::Model;

pub fn parse_args() -> Config {
    let mut args = env::args();
    args.next();

    let model_path = args
        .next()
        .expect("A model path(ex: vosk-model-small-ja-0.22) was not provided");
    let record_duration = Duration::from_secs(
        args.next()
            .expect("A recording duration(ex: 10) was not provided")
            .parse()
            .expect("Invalid recording duration"),
    );

    let audio_input_device = cpal::default_host()
        .default_input_device()
        .expect("No input device connected");

    let supported_stream_config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = supported_stream_config.channels();

    let model = Model::new(model_path).expect("Could not create the model");

    Config {
        model,
        record_duration,
        supported_stream_config,
        audio_input_device,
        channels,
    }
}
