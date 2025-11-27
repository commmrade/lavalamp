use std::sync::{Arc, Mutex};

use axum::Router;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

mod error;
mod handlers;
mod stream;

#[derive(Clone)]
struct AppState {
    // cam: Arc<Mutex<opencv::videoio::VideoCapture>>,
    prev_hash: [u8; 32],
    last_frame: Arc<Mutex<opencv::core::Mat>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // dotenv::dotenv()?; // comment when build for docker

    let state = AppState {
        prev_hash: [0u8; 32],
        last_frame: Arc::new(Mutex::new(opencv::core::Mat::default())),
    };

    let last_frame = state.last_frame.clone();
    tokio::spawn(async move { stream::stream_loop(last_frame) });

    let (router, api) = OpenApiRouter::new()
        .routes(routes!(handlers::hash::generate_hash))
        .split_for_parts();
    let app = Router::new()
        .merge(router)
        .with_state(state)
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", api));

    let listener = tokio::net::TcpListener::bind(std::env::var("ADDR")?).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
