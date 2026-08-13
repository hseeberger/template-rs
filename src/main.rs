{%- if config and otel -%}
use anyhow::Context;
use configured::{Case, Configured};
use opentelemetry::{InstrumentationScope, global, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use serde::Deserialize;
use serde_json::json;
use std::panic;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
{%- elsif config -%}
use anyhow::Context;
use configured::{Case, Configured};
use serde::Deserialize;
use serde_json::json;
use std::panic;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
{%- elsif otel -%}
use anyhow::Context;
use opentelemetry::{InstrumentationScope, global, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use serde::Deserialize;
use serde_json::json;
use std::panic;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
{%- else -%}
use std::panic;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
{%- endif %}

#[tokio::main]
async fn main() {
{%- if config %}
    let Ok(config) = Config::load(Case::Snake)
        .context("load configuration")
        .inspect_err(log_error)
    else {
        return;
    };
{% endif %}
{%- if otel %}
{%- if config %}
    let Ok(provider) = init_tracing(config.tracing.clone()).inspect_err(log_error) else {
{%- else %}
    let Ok(provider) = init_tracing(TracingConfig::default()).inspect_err(log_error) else {
{%- endif %}
        return;
    };
{%- else %}
    init_tracing();
{%- endif %}

    panic::set_hook(Box::new(|panic| error!(%panic, "process panicked")));

{%- if config %}

    if let Err(error) = run(config).await {
{%- else %}

    if let Err(error) = run().await {
{%- endif %}
        let backtrace = error.backtrace();
        let error = format!("{error:#}");
        error!(error, %backtrace, "process exited with ERROR")
    }
{%- if otel %}

    if let Some(provider) = provider
        && let Err(error) = provider.shutdown()
    {
        error!(%error, "cannot shut down tracer provider")
    }
{%- endif %}
}
{%- if config %}

{%- if otel %}
#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "tracing", default)]
    tracing: TracingConfig,
}
{%- else %}
#[derive(Debug, Deserialize)]
struct Config {}
{%- endif %}
{%- endif %}
{%- if otel %}

#[derive(Debug, Clone, Deserialize)]
struct TracingConfig {
    #[serde(default)]
    enabled: bool,

    #[serde(default = "otlp_exporter_endpoint_default")]
    otlp_exporter_endpoint: String,

    #[serde(default = "package_name")]
    service_name: String,

    #[serde(default = "package_name")]
    instrumentation_scope_name: String,

    #[serde(default = "package_version")]
    instrumentation_scope_version: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            otlp_exporter_endpoint: otlp_exporter_endpoint_default(),
            service_name: package_name(),
            instrumentation_scope_name: package_name(),
            instrumentation_scope_version: package_version(),
        }
    }
}
{%- endif %}
{%- if config or otel %}

fn log_error(error: &anyhow::Error) {
    let error = json!({
        "level": "ERROR",
        "message": "process exited with ERROR",
        "error": format!("{error:#}"),
    });
    println!("{error}");
}
{%- endif %}
{%- if otel %}

fn init_tracing(config: TracingConfig) -> anyhow::Result<Option<SdkTracerProvider>> {
    let TracingConfig {
        enabled,
        otlp_exporter_endpoint,
        service_name,
        instrumentation_scope_name,
        instrumentation_scope_version,
    } = config;

    let provider = enabled
        .then(|| tracer_provider(otlp_exporter_endpoint, service_name))
        .transpose()?;

    let otlp_layer = provider.as_ref().map(|provider| {
        let scope = InstrumentationScope::builder(instrumentation_scope_name)
            .with_version(instrumentation_scope_version)
            .build();
        tracing_opentelemetry::layer().with_tracer(provider.tracer_with_scope(scope))
    });

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .with(otlp_layer)
        .try_init()
        .context("initialize tracing subscriber")?;

    Ok(provider)
}
{%- else %}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .init();
}
{%- endif %}

{% if config -%}
async fn run(config: Config) -> anyhow::Result<()> {
    info!(?config, "starting");
{%- else -%}
async fn run() -> anyhow::Result<()> {
    info!("starting");
{%- endif %}

    Ok(())
}
{%- if otel %}

fn tracer_provider(
    otlp_exporter_endpoint: String,
    service_name: String,
) -> anyhow::Result<SdkTracerProvider> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_exporter_endpoint)
        .build()
        .context("build OTLP exporter")?;

    let resource = Resource::builder().with_service_name(service_name).build();

    global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

fn otlp_exporter_endpoint_default() -> String {
    "http://localhost:4317".into()
}

fn package_name() -> String {
    env!("CARGO_PKG_NAME").to_owned()
}

fn package_version() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}
{%- endif %}
