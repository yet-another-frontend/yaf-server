use crate::recognizer::{Recognizer, STTModel};

pub struct AppState {
    pub recognizer: Recognizer,
}

impl AppState {
    pub fn new(model: STTModel) -> Self {
        Self {
            recognizer: Recognizer::new(model).expect("Failed to initialize recognizer"),
        }
    }
}
