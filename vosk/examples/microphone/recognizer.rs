use super::{models::Config, stream::create_stream};
use cpal::{traits::StreamTrait, ChannelCount};
use dasp::{sample::ToSample, Sample};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use vosk::{DecodingState, Recognizer};

pub fn start_recognition(
    recognizer: Arc<Mutex<Recognizer>>,
    current_text: Arc<Mutex<String>>,
    prev_partial: Arc<Mutex<String>>,
    config: Config,
) -> (cpal::Stream, Duration) {
    let recognizer_clone = recognizer.clone();
    let current_text_clone = current_text.clone();
    let prev_partial_clone = prev_partial.clone();

    let record_duration = config.record_duration;

    let stream = create_stream(
        config,
        recognizer_clone,
        current_text_clone,
        prev_partial_clone,
    )
    .expect("Could not build stream");
    stream.play().expect("Could not play stream");
    println!("Recording...");

    (stream, record_duration)
}

pub fn recognize<T: Sample + ToSample<i16>>(
    recognizer: &mut Recognizer,
    data: &[T],
    channels: ChannelCount,
    current_text: &mut String,
    prev_partial: &mut String,
) {
    let data: Vec<i16> = data.iter().map(|v| v.to_sample()).collect();
    let data = if channels != 1 {
        stereo_to_mono(&data)
    } else {
        data
    };

    let state = recognizer.accept_waveform(&data);
    match state {
        DecodingState::Running => {
            // println!("partial: {:#?}", recognizer.partial_result());
            let partial_result = recognizer.partial_result().partial;
            if partial_result != prev_partial {
                *prev_partial = partial_result.to_string();
                println!("partial: {}", *prev_partial);
            }
        }
        DecodingState::Finalized => {
            // Result will always be multiple because we called set_max_alternatives
            // println!("result: {:#?}", recognizer.result().multiple().unwrap());
            let final_result_multiple = recognizer.result().multiple().unwrap();
            match final_result_multiple.alternatives.iter().next() {
                Some(alternative) => {
                    *current_text =
                        (*current_text).as_str().to_string() + &alternative.text.to_string();
                    println!("final: {}", current_text);
                }
                _ => (),
            };
        }
        DecodingState::Failed => eprintln!("error"),
    }
}

pub fn update_settings(recognizer: &mut Recognizer, max_alternatives: u16) {
    recognizer.set_max_alternatives(max_alternatives);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);
}

fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
}
