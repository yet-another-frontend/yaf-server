use crate::{config::ServerConfig, recognizer::Recognizer};

pub struct AppState {
    pub recognizer: Recognizer,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            recognizer: Recognizer::new(config).expect("Failed to initialize recognizer"),
        }
    }
}
