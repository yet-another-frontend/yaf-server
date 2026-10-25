use crate::recognizer::RecognizerError;

pub mod transcribe;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("recognizer failed: {0}")]
    RecognizerError(#[from] RecognizerError),

    #[error("blocking task failed")]
    JoinError(#[from] tokio::task::JoinError),
}
