use crate::{handlers::ServerError, recognizer::RecognizerError, state::AppState};
use axum::http::StatusCode;
use std::sync::Arc;
use tokio::task;

pub async fn transcribe_audio(
    state: Arc<AppState>,
    audio_samples: Vec<f32>,
    sample_rate: i32,
) -> Result<String, (StatusCode, ServerError)> {
    if sample_rate <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            RecognizerError::InvalidSampleRate.into(),
        ));
    }

    let result =
        task::spawn_blocking(move || state.recognizer.transcribe(&audio_samples, sample_rate))
            .await;

    match result {
        Ok(Ok(text)) => Ok(text),

        Ok(Err(err)) => Err((StatusCode::BAD_REQUEST, err.into())),

        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.into())),
    }
}
