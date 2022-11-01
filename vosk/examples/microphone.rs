//! Run with:
//! cargo run --example microphone <model path> <duration>
//! e.g. "cargo run --example microphone /home/user/stt/model 10"
//!
//! Read the "Setup" section in the README to know how to link the vosk dynamic
//! libaries to the examples

use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ChannelCount, SampleFormat,
};
use dasp::{sample::ToSample, Sample};
use vosk::{DecodingState, Model, Recognizer};

fn main() {
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

    let config = audio_input_device
        .default_input_config()
        .expect("Failed to load default input config");
    let channels = config.channels();

    let model = Model::new(model_path).expect("Could not create the model");
    let mut recognizer = Recognizer::new(&model, config.sample_rate().0 as f32)
        .expect("Could not create the Recognizer");

    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);

    let recognizer = Arc::new(Mutex::new(recognizer));

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let current_text = Arc::new(Mutex::new(String::new()));
    let prev_partial = Arc::new(Mutex::new(String::new()));

    let recognizer_clone = recognizer.clone();
    let current_text_clone = current_text.clone();
    let prev_partial_clone = prev_partial.clone();
    let stream = match config.sample_format() {
        SampleFormat::F32 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[f32], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
        SampleFormat::U16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[u16], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
        SampleFormat::I16 => audio_input_device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                recognize(
                    &mut recognizer_clone.lock().unwrap(),
                    data,
                    channels,
                    &mut current_text_clone.lock().unwrap(),
                    &mut prev_partial_clone.lock().unwrap(),
                )
            },
            err_fn,
        ),
    }
    .expect("Could not build stream");

    stream.play().expect("Could not play stream");
    println!("Recording...");

    std::thread::sleep(record_duration);
    drop(stream);

    // println!("{:#?}", recognizer.lock().unwrap().final_result());
}

fn recognize<T: Sample + ToSample<i16>>(
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

pub fn stereo_to_mono(input_data: &[i16]) -> Vec<i16> {
    let mut result = Vec::with_capacity(input_data.len() / 2);
    result.extend(
        input_data
            .chunks_exact(2)
            .map(|chunk| chunk[0] / 2 + chunk[1] / 2),
    );

    result
}
