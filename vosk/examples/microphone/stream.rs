use super::{models::Config, recognizer::recognize};
use cpal::{traits::DeviceTrait, SampleFormat};
use std::sync::{Arc, Mutex};
use vosk::Recognizer;

pub fn create_stream(
    config: Config,
    recognizer_clone: Arc<Mutex<Recognizer>>,
    current_text_clone: Arc<Mutex<String>>,
    prev_partial_clone: Arc<Mutex<String>>,
) -> Result<cpal::Stream, cpal::BuildStreamError> {
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    match config.supported_stream_config.sample_format() {
        SampleFormat::F32 => config.audio_input_device.build_input_stream(
            &config.supported_stream_config.into(),
            move |data: &[f32], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    config.channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
        SampleFormat::U16 => config.audio_input_device.build_input_stream(
            &config.supported_stream_config.into(),
            move |data: &[u16], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    config.channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
        SampleFormat::I16 => config.audio_input_device.build_input_stream(
            &config.supported_stream_config.into(),
            move |data: &[i16], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    config.channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
    }
}
