use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Cpu,
    Cuda,
    Trt,
    Coreml,
    Qnn,
    Rknn,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Cpu => "cpu",
            Provider::Cuda => "cuda",
            Provider::Trt => "trt",
            Provider::Coreml => "coreml",
            Provider::Qnn => "qnn",
            Provider::Rknn => "rknn",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types, unused)]
pub enum STTModel {
    #[serde(rename = "whisper-large-v3-turbo")]
    WhisperLargeV3Turbo,

    #[serde(rename = "whisper-large-v3")]
    WhisperLargeV3,

    #[serde(rename = "whisper-large-v2")]
    WhisperLargeV2,

    #[serde(rename = "whisper-large")]
    WhisperLarge,

    #[serde(rename = "whisper-medium")]
    WhisperMedium,

    #[serde(rename = "whisper-small")]
    WhisperSmall,

    #[serde(rename = "whisper-base")]
    WhisperBase,

    #[serde(rename = "whisper-tiny")]
    WhisperTiny,
}

impl STTModel {
    pub fn model_name(&self) -> &str {
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

    pub fn archive_name(&self) -> String {
        format!("sherpa-onnx-whisper-{}.tar.bz2", self.model_name())
    }

    pub fn download_url(&self) -> String {
        format!(
            "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/{}",
            self.archive_name()
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub model: STTModel,
    pub models_dir: PathBuf,
    pub port: u16,
    pub provider: Provider,
}

#[derive(thiserror::Error, Debug)]
pub enum ServerConfigError {
    #[error("invalid config format: {0}")]
    InvalidFormat(#[from] serde_json::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ServerConfig {
    pub fn read(path: PathBuf) -> Result<Self, ServerConfigError> {
        let json = std::fs::read_to_string(path)?;
        let config: ServerConfig = serde_json::from_str(&json)?;

        Ok(config)
    }
}
