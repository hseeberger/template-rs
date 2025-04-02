mod config;
mod error;
mod infra;
mod telemetry;

use crate::{
    config::{ConfigExt, MainConfig},
    infra::api,
};
use anyhow::Context;
use log::{error, info};
use std::panic;

/// The entry point into the application.
#[tokio::main]
pub async fn main() {
    // Initialize logging.
    telemetry::init_logging();

    // Replace the default panic hook with one that uses structured logging at ERROR level.
    panic::set_hook(Box::new(|panic| error!(panic:%; "process panicked")));

    // Run and log any error.
    if let Err(error) = run().await {
        let backtrace = error.backtrace();
        let error = format!("{error:#}");
        error!(error, backtrace:%; "process exited with ERROR")
    }
}

async fn run() -> anyhow::Result<()> {
    // Load configuration.
    let MainConfig {
        config,
        tracing_config,
    } = MainConfig::load().context("load configuration")?;

    // Initialize tracing.
    telemetry::init_tracing(tracing_config);

    info!(config:?; "starting");

    api::serve(config.api).await
}
