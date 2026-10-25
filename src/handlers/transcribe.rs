use crate::{handlers::utils::transcribe_audio, state::AppState};
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct TranscribeRequest {
    pub audio_samples: Vec<f32>,
    pub sample_rate: i32,
}

pub async fn transcribe(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TranscribeRequest>,
) -> (StatusCode, Json<String>) {
    let client_ip = addr.ip();
    let client_port = addr.port();

    match transcribe_audio(
        Arc::clone(&state),
        payload.audio_samples,
        payload.sample_rate,
    )
    .await
    {
        Ok(text) => {
            info!(
                client_ip = %client_ip,
                client_port = client_port,
                status = 200,
                "Successful transcription!"
            );

            (StatusCode::OK, Json(text))
        }
        Err((status, err)) => {
            warn!(
                error = err.to_string(),
                client_ip = %client_ip,
                client_port = client_port,
                status = status.as_u16(),
                "Transcription request failed"
            );

            (status, Json(err.to_string()))
        }
    }
}
