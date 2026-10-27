mod config;
mod handlers;
mod recognizer;
mod state;

use crate::config::ServerConfig;
use crate::handlers::transcribe::transcribe;
use crate::state::AppState;
use axum::{Router, routing::post};
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

fn main() {
    tracing_subscriber::fmt().init();

    // Synchronous runtime is necessary to download models
    let config = ServerConfig::read("./config.json".into()).unwrap();
    let addr = format!("0.0.0.0:{}", config.port);
    let state = Arc::new(AppState::new(config));

    // Start tokio asynchronous runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = Router::new()
            .route("/test/transcribe", post(transcribe))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        info!("Listening on {}.", &addr);

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
}
