mod config;
mod error;
mod telemetry;

use crate::{
    config::{Config, ConfigExt, MainConfig},
    error::log_error,
};
use anyhow::{Context, Result};
use std::panic;
use tracing::{error, info};

/// The entry point into the application.
pub async fn main() -> Result<()> {
    // Load configuration first, because needed for tracing initialization.
    let MainConfig {
        config,
        telemetry_config,
    } = MainConfig::load()
        .context("load configuration")
        .inspect_err(log_error)?;

    // Initialize tracing.
    telemetry::init(telemetry_config).inspect_err(log_error)?;

    // Replace the default panic hook with one that uses structured logging at ERROR level.
    panic::set_hook(Box::new(|panic| error!(%panic, "process panicked")));

    // Run and log any error.
    run(config).await.inspect_err(|error| {
        error!(
            error = format!("{error:#}"),
            backtrace = %error.backtrace(),
            "process exited with ERROR"
        )
    })
}

async fn run(config: Config) -> Result<()> {
    info!(?config, "starting");

    // Application code here ...
    Ok(())
}
