use crate::info;
use bzip2::read::BzDecoder;
use rubato::audioadapter_buffers::owned::InterleavedOwned;
use rubato::{Fft, FixedSync, Resampler};
use sherpa_onnx::{OfflineRecognizer, OfflineRecognizerConfig, OfflineWhisperModelConfig};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tar::Archive;

const MODELS_PATH: &str = "./models";
const TARGET_SAMPLE_RATE: i32 = 16_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, unused)]
pub enum STTModel {
    WhisperLargeV3Turbo,
    WhisperLargeV3,
    WhisperLargeV2,
    WhisperLarge,
    WhisperMedium,
    WhisperSmall,
    WhisperBase,
    WhisperTiny,
}

impl STTModel {
    fn model_name(&self) -> &str {
        match self {
            Self::WhisperLargeV3Turbo => "turbo",
            Self::WhisperLargeV3 => "large-v3",
            Self::WhisperLargeV2 => "large-v2",
            Self::WhisperLarge => "large",
            Self::WhisperMedium => "medium",
            Self::WhisperSmall => "small",
            Self::WhisperBase => "base",
            Self::WhisperTiny => "tiny",
        }
    }

    fn archive_name(&self) -> String {
        format!("sherpa-onnx-whisper-{}.tar.bz2", self.model_name())
    }

    pub fn download_url(&self) -> String {
        format!(
            "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/{}",
            self.archive_name()
        )
    }
}

pub struct Recognizer {
    recognizer: OfflineRecognizer,
    _recognizer_config: OfflineRecognizerConfig,
}

#[derive(thiserror::Error, Debug)]
pub enum RecognizerError {
    #[error("invalid sample rate")]
    InvalidSampleRate,

    #[error("transcription failed: {0}")]
    TranscriptionFail(String),

    #[error("model download failed: {0}")]
    DownloadModel(#[from] DownloadModelError),
}

#[derive(thiserror::Error, Debug)]
pub enum DownloadModelError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("expected encoder file was not found in {path}")]
    EncoderNotFound { path: PathBuf },
}

impl Recognizer {
    pub fn new(model: STTModel) -> Result<Self, RecognizerError> {
        let mut recognizer_config = OfflineRecognizerConfig::default();
        recognizer_config.model_config.provider = Some("cpu".to_string());

        let model_dir = download_model(model, MODELS_PATH)?;
        let model_name = model.model_name();

        recognizer_config.model_config.whisper = OfflineWhisperModelConfig {
            encoder: Some(
                model_dir
                    .join(format!("{model_name}-encoder.onnx"))
                    .to_string_lossy()
                    .into_owned(),
            ),
            decoder: Some(
                model_dir
                    .join(format!("{model_name}-decoder.onnx"))
                    .to_string_lossy()
                    .into_owned(),
            ),
            language: Some("en".to_string()),
            task: Some("transcribe".to_string()),
            tail_paddings: 0,
            enable_token_timestamps: false,
            enable_segment_timestamps: false,
        };

        recognizer_config.model_config.tokens = Some(
            model_dir
                .join(format!("{model_name}-tokens.txt"))
                .to_string_lossy()
                .into_owned(),
        );

        let recognizer =
            OfflineRecognizer::create(&recognizer_config).expect("failed to create recognizer");

        info!(
            "Recognizer initialized with Whisper {} model.",
            model.model_name()
        );

        Ok(Self {
            recognizer,
            _recognizer_config: recognizer_config,
        })
    }

    pub fn transcribe(&self, samples: &[f32], sample_rate: i32) -> Result<String, RecognizerError> {
        if samples.is_empty() {
            return Ok(String::new());
        }

        if sample_rate <= 0 {
            return Err(RecognizerError::InvalidSampleRate);
        }

        let resampled_samples = resample_audio(samples, sample_rate)
            .map_err(|e| RecognizerError::TranscriptionFail(e.to_string()))?;

        let stream = self.recognizer.create_stream();

        stream.accept_waveform(TARGET_SAMPLE_RATE, &resampled_samples);

        self.recognizer.decode(&stream);

        let result = stream.get_result().ok_or_else(|| {
            RecognizerError::TranscriptionFail("failed to retrieve recognition result".to_string())
        })?;

        Ok(result.text)
    }
}

fn resample_audio(
    samples: &[f32],
    current_sample_rate: i32,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    if current_sample_rate == TARGET_SAMPLE_RATE {
        return Ok(samples.to_vec());
    }

    let mut resampler = Fft::<f32>::new(
        current_sample_rate as usize,
        TARGET_SAMPLE_RATE as usize,
        1024,
        1, // Mono audio
        FixedSync::Both,
    )?;

    let input = InterleavedOwned::new_from(samples.to_vec(), 1, samples.len())?;
    let output = resampler.process_all(&input, samples.len(), None)?;

    Ok(output.take_data())
}

fn download_model(
    model: STTModel,
    cache_dir: impl AsRef<Path>,
) -> Result<PathBuf, DownloadModelError> {
    // Create directory to store model in
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir)?;
    let model_dir = cache_dir.join(format!("sherpa-onnx-whisper-{}", model.model_name()));

    if model_dir
        .join(format!("{}-encoder.onnx", model.model_name()))
        .exists()
    {
        info!(
            "{} already downloaded.",
            model_dir.file_name().unwrap().to_string_lossy()
        );
        return Ok(model_dir);
    }

    // Create download file for model archive
    info!(
        "Downloading {}...",
        model_dir.file_name().unwrap().to_string_lossy()
    );
    let archive_path = cache_dir.join(model.archive_name());
    let response = reqwest::blocking::Client::new()
        .get(model.download_url())
        .send()?
        .error_for_status()?;

    // Use temp file to handle download interruptions
    let temp_path = cache_dir.join("whisper-download.tmp");
    {
        let mut output = File::create(&temp_path)?;
        let mut response = response;
        std::io::copy(&mut response, &mut output)?;
        output.sync_all()?;
    }
    fs::rename(&temp_path, &archive_path)?;

    // Unzip archive file
    let archive_file = File::open(&archive_path)?;
    let decoder = BzDecoder::new(archive_file);
    let mut archive = Archive::new(decoder);

    archive.unpack(cache_dir)?;

    if !model_dir
        .join(format!("{}-encoder.onnx", model.model_name()))
        .exists()
    {
        return Err(DownloadModelError::EncoderNotFound {
            path: model_dir.clone(),
        });
    }

    fs::remove_file(&archive_path)?;

    info!(
        "Finished downloading {}.",
        model_dir.file_name().unwrap().to_string_lossy()
    );
    Ok(model_dir)
}
