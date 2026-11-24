mod ai;
mod ffmpeg;
mod jobs;
mod routes;
mod state;

use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let ffmpeg_path = env::var("FFMPEG_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "ffmpeg".to_string());
    let ffprobe_path = env::var("FFPROBE_PATH")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "ffprobe".to_string());
    let ai_upscaler_bin = env::var("AI_UPSCALER_BIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let ffmpeg_threads = env::var("FFMPEG_THREADS")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "2".to_string());
    let max_output_pixels = env::var("MAX_OUTPUT_PIXELS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value >= 1_000_000)
        .unwrap_or(5_000_000);

    let data_dir = state::resolve_data_dir().await;
    let loaded_jobs = jobs::load_jobs(&data_dir).await;
    let encoders = ffmpeg::detect_encoders(&ffmpeg_path).await;
    let fontfile = state::find_fontfile();

    info!(
        ?data_dir, ?ffmpeg_path, ?ffprobe_path, ?fontfile,
        ffmpeg_threads = %ffmpeg_threads,
        max_output_pixels,
        "VidBoost server starting"
    );

    let app_state = Arc::new(state::AppState {
        jobs: Arc::new(Mutex::new(loaded_jobs)),
        data_dir: data_dir.clone(),
        ffmpeg_path,
        ffprobe_path,
        ai_upscaler_bin,
        fontfile,
        encoders: Arc::new(encoders),
        children: Arc::new(Mutex::new(HashMap::new())),
        ffmpeg_threads,
        max_output_pixels,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/status", get(routes::status))
        .route("/ai/models", get(routes::ai_models))
        .route("/video/upload", post(routes::upload))
        .route("/video/logo", post(routes::logo))
        .route("/video/process", post(routes::process))
        .route("/video/cancel", post(routes::cancel))
        .route("/video/job", get(routes::job))
        .route("/video/source", get(routes::source))
        .route("/video/download", get(routes::download))
        .route("/video/thumbnail", get(routes::thumbnail));

    let app = Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024))
        .with_state(app_state);

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(8080);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address).await?;

    info!("VidBoost API listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}
