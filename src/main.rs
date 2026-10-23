pub mod macros;
mod recognizer;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use recognizer::{Recognizer, STTModel};
use std::io;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recognizer = Recognizer::new(STTModel::WhisperBase)?;

    // Find the default microphone.
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("No microphone found")?;
    let config = device.default_input_config()?;
    let sample_rate = config.sample_rate() as i32;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();

    // Shared buffer for microphone samples.
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let recording_samples = Arc::clone(&samples);

    // Build the microphone input stream.
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buffer) = recording_samples.lock() {
                    buffer.extend_from_slice(data);
                }
            },
            |err| eprintln!("Microphone error: {err}"),
            None,
        )?,

        _ => return Err(format!("Unsupported microphone format: {sample_format:?}").into()),
    };

    // Start recording.
    stream.play()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Stop recording before reading the collected samples.
    drop(stream);

    let recorded_samples = samples
        .lock()
        .map_err(|e| format!("Failed to access audio buffer: {e}"))?
        .clone();

    if recorded_samples.is_empty() {
        println!("No audio was recorded.");
        return Ok(());
    }

    // Convert interleaved audio channels to mono.
    let mono_samples: Vec<f32> = recorded_samples
        .chunks(channels)
        .filter(|frame| frame.len() == channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();

    println!("Recording stopped. Transcribing...");

    // Pass the microphone samples into your existing transcribe method.
    let transcription = recognizer.transcribe(&mono_samples, sample_rate)?;

    println!("Transcription: {transcription}");

    Ok(())
}
