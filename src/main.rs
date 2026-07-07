{%- if config and tracing -%}
use anyhow::Context;
use configured::{Case, Configured};
use fastrace_opentelemetry::OpenTelemetryReporter;
use log::{error, info};
use logforth::{
    append::{FastraceEvent, Stdout},
    diagnostic::FastraceDiagnostic,
    filter::rustlog::RustLogFilterBuilder,
    layout::JsonLayout,
};
use opentelemetry::InstrumentationScope;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use serde::Deserialize;
use std::{borrow::Cow, panic};
use tokio::runtime::Handle;
{%- elsif config -%}
use anyhow::Context;
use configured::{Case, Configured};
use log::{error, info};
use logforth::{append::Stdout, filter::rustlog::RustLogFilterBuilder, layout::JsonLayout};
use serde::Deserialize;
use std::panic;
{%- elsif tracing -%}
use anyhow::Context;
use fastrace_opentelemetry::OpenTelemetryReporter;
use log::{error, info};
use logforth::{
    append::{FastraceEvent, Stdout},
    diagnostic::FastraceDiagnostic,
    filter::rustlog::RustLogFilterBuilder,
    layout::JsonLayout,
};
use opentelemetry::InstrumentationScope;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use serde::Deserialize;
use std::{borrow::Cow, panic};
use tokio::runtime::Handle;
{%- else -%}
use log::{error, info};
use logforth::{append::Stdout, filter::rustlog::RustLogFilterBuilder, layout::JsonLayout};
use std::panic;
{%- endif %}

#[tokio::main]
async fn main() {
    init_logging();

    // Replace the default panic hook with one that uses structured logging at ERROR level.
    panic::set_hook(Box::new(|panic| error!(panic:%; "process panicked")));

    // Run and log any error.
    if let Err(error) = run().await {
        let backtrace = error.backtrace();
        let error = format!("{error:#}");
        error!(error, backtrace:%; "process exited with ERROR")
    }
{%- if tracing %}

    // Drain the batching trace reporter so the tail of spans is exported before exit.
    fastrace::flush();
{%- endif %}
}
{%- if config %}

{%- if tracing %}
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
{%- if tracing %}

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

async fn run() -> anyhow::Result<()> {
{%- if config %}
    let config = Config::load(Case::Snake).context("load configuration")?;
    info!(config:?; "starting");
{%- else %}
    info!("starting");
{%- endif %}
{%- if tracing %}
{%- if config %}
    init_tracing(config.tracing).context("initialize tracing")?;
{%- else %}
    init_tracing(TracingConfig::default()).context("initialize tracing")?;
{%- endif %}
{%- endif %}

    Ok(())
}

fn init_logging() {
    logforth::starter_log::builder()
        .dispatch(|dispatch| {
            dispatch
                .filter(RustLogFilterBuilder::from_default_env().build())
{%- if tracing %}
                .diagnostic(FastraceDiagnostic::default())
{%- endif %}
                .append(Stdout::default().with_layout(JsonLayout::default()))
{%- if tracing %}
                .append(FastraceEvent::default())
{%- endif %}
        })
        .apply();
}
{%- if tracing %}

fn init_tracing(config: TracingConfig) -> anyhow::Result<()> {
    if config.enabled {
        let TracingConfig {
            otlp_exporter_endpoint,
            service_name,
            instrumentation_scope_name,
            instrumentation_scope_version,
            ..
        } = config;

        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_exporter_endpoint)
            .build()
            .context("build OTLP exporter")?;

        let resource = Resource::builder().with_service_name(service_name).build();

        let instrumentation_scope = InstrumentationScope::builder(instrumentation_scope_name)
            .with_version(instrumentation_scope_version)
            .build();

        // The OTLP/tonic exporter is async; fastrace's default `block_on` cannot drive it, so hand
        // it this runtime's handle. Requires `init_tracing` to be called from within the runtime.
        let handle = Handle::current();
        let reporter =
            OpenTelemetryReporter::new(exporter, Cow::Owned(resource), instrumentation_scope)
                .with_block_on(move |future| handle.block_on(future));

        fastrace::set_reporter(reporter, fastrace::collector::Config::default());
    }

    Ok(())
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
