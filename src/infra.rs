pub mod api;

use serde::Deserialize;

/// Infra configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "api")]
    pub api_config: api::Config,
}
