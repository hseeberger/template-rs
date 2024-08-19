mod config;

use crate::config::{Config, ConfigExt};
use anyhow::{Context, Result};
use serde_json::json;
use std::{fmt::Display, panic};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing.
    init_tracing().inspect_err(log_error)?;

    // Load configuration.
    let config = Config::load().context("load configuration")?;

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

/// Initialize tracing: apply an `EnvFilter` using the `RUST_LOG` environment variable to define the
/// log levels and add a formatter layer logging lopg events as JSON.
fn init_tracing() -> Result<()> {
    // global::set_text_map_propagator(TraceContextPropagator::new());

    // global::set_error_handler(|error| error!(error = error.as_chain(), "otel error"))
    //     .context("set error handler")?;

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().json().flatten_event(true))
        .try_init()
        .context("initialize tracing subscriber")
}

fn log_error(error: &impl Display) {
    let now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
    let error = serde_json::to_string(&json!({
        "timestamp": now,
        "level": "ERROR",
        "message": "process exited with ERROR",
        "error": format!("{error:#}")
    }));
    // Not using `eprintln!`, because `tracing_subscriber::fmt` uses stdout by default.
    println!("{}", error.expect("error can be serialized to JSON"));
}

async fn run(config: Config) -> Result<()> {
    info!(?config, "starting");

    // Your code here ...
    Ok(())
}
