use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub label: String,
    pub image: String,
    pub arguments: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub gateways: BTreeMap<String, GatewayConfig>,
}
