use anyhow::Context;
use axum::{Router, response::IntoResponse, routing::get};
use fastrace::trace;
use fastrace_axum::FastraceLayer;
use serde::Deserialize;
use std::net::IpAddr;
use tokio::{
    net::TcpListener,
    signal::unix::{SignalKind, signal},
};
use tower::ServiceBuilder;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub address: IpAddr,
    pub port: u16,
}

pub async fn serve(config: Config) -> anyhow::Result<()> {
    let Config { address, port } = config;

    let app = app().layer(ServiceBuilder::new().layer(FastraceLayer));

    let listener = TcpListener::bind((address, port))
        .await
        .context("bind TcpListener")?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("run server")
}

pub fn app() -> Router {
    Router::new().route("/", get(ready))
}

#[trace]
async fn ready() -> impl IntoResponse {
    "ready"
}

async fn shutdown_signal() {
    signal(SignalKind::terminate())
        .expect("install SIGTERM handler")
        .recv()
        .await;
}
