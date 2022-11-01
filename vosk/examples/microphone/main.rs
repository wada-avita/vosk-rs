//! Run with:
//! cargo run --example microphone <model path> <duration>
//! e.g. "cargo run --example microphone /home/user/stt/model 10"
//!
//! Read the "Setup" section in the README to know how to link the vosk dynamic
//! libaries to the examples
mod models;
mod recognizer;
mod stream;
mod utils;

use recognizer::{start_recognition, update_settings};
use std::sync::{Arc, Mutex};
use utils::parse_args;
use vosk::Recognizer;

fn main() {
    let config = parse_args();
    let mut recognizer = Recognizer::new(
        &config.model,
        config.supported_stream_config.sample_rate().0 as f32,
    )
    .expect("Could not create the Recognizer");
    let max_alternatives = 10;
    update_settings(&mut recognizer, max_alternatives);

    let recognizer = Arc::new(Mutex::new(recognizer));
    let current_text = Arc::new(Mutex::new(String::new()));
    let prev_partial = Arc::new(Mutex::new(String::new()));

    let (stream, record_duration) =
        start_recognition(recognizer, current_text, prev_partial, config);

    std::thread::sleep(record_duration);
    drop(stream);

    // println!("{:#?}", recognizer.lock().unwrap().final_result());
}
