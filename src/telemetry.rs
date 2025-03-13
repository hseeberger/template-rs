use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider, Resource};
use serde::Deserialize;
use thiserror::Error;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Telemetry (logging, tracing, metrics) configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "tracing")]
    pub tracing_config: TracingConfig,
}

/// Tracing (as opposed to logging or metrics) configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,

    #[serde(default = "otlp_exporter_endpoint_default")]
    pub otlp_exporter_endpoint: String,

    #[serde(default)]
    pub service_name: Option<String>,
}

/// Error possibly returned by [init].
#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot initialize tracing subscriber")]
    TryInit(#[from] tracing_subscriber::util::TryInitError),

    #[error("cannot install OTLP tracer")]
    InstallOtlpTracer(#[from] opentelemetry::trace::TraceError),
}

/// Initialize telemetry: apply an `EnvFilter` using the `RUST_LOG` environment variable to define
/// the log levels, add a formatter layer logging as JSON and an OpenTelemetry layer exporting
/// tracing data if tracing is enabled.
pub fn init(config: Config) -> Result<(), Error> {
    let Config { tracing_config } = config;

    let tracing = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true));

    // The below little code duplication is needed because `tracing` and
    // `tracing.with(otlp_layer(config)?)` have different types.
    if tracing_config.enabled {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        tracing.with(otlp_layer(tracing_config)?).try_init()?
    } else {
        tracing.try_init()?
    }

    Ok(())
}

/// Create an OTLP layer exporting tracing data.
fn otlp_layer<S>(config: TracingConfig) -> Result<impl tracing_subscriber::Layer<S>, Error>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(config.otlp_exporter_endpoint)
        .build()?;

    let mut resource = Resource::builder();
    if let Some(service_name) = config.service_name {
        resource = resource.with_service_name(service_name);
    }

    let provider = SdkTracerProvider::builder()
        .with_resource(resource.build())
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer("config.service_name");

    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}

fn otlp_exporter_endpoint_default() -> String {
    "http://localhost:4317".to_string()
}

#[cfg(test)]
mod tests {
    use crate::telemetry::{self, Config, TracingConfig};

    #[tokio::test]
    async fn test_init() {
        let tracing_config = TracingConfig {
            enabled: true,
            otlp_exporter_endpoint: "http://localhost:4317".to_string(),
            service_name: None,
        };
        let config = Config { tracing_config };
        let result = telemetry::init(config);
        assert!(result.is_ok());

        let tracing_config = TracingConfig {
            enabled: false,
            otlp_exporter_endpoint: "http://localhost:4317".to_string(),
            service_name: None,
        };
        let config = Config { tracing_config };
        let result = telemetry::init(config);
        assert!(result.is_err());
    }
}
