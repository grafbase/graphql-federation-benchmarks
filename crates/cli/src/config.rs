use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::gateway::Gateway;

/// Central configuration for the entire benchmark repository
pub struct Config {
    pub scenarios: BTreeMap<String, ScenarioConfig>,
    pub supergraphs: BTreeMap<String, SupergraphConfig>,
    pub gateways: Vec<Arc<Gateway>>,
    pub current_dir: PathBuf,
}

impl Config {
    /// Load all configurations from the repository
    pub fn load(current_dir: PathBuf) -> Result<Self> {
        // Load scenarios
        let scenarios = load_scenarios(&current_dir)?;

        // Load supergraphs
        let supergraphs = load_supergraphs(&current_dir)?;

        // Load gateways
        let gateways = crate::gateway::load(&current_dir, None)?;

        Ok(Self {
            scenarios,
            supergraphs,
            gateways,
            current_dir,
        })
    }

    pub fn get_scenario(&self, name: &str) -> Result<&ScenarioConfig> {
        self.scenarios.get(name).ok_or_else(|| {
            anyhow::anyhow!("Scenario '{}' not found in scenarios/config.toml", name)
        })
    }

    pub fn get_supergraph(&self, name: &str) -> Result<&SupergraphConfig> {
        self.supergraphs.get(name).ok_or_else(|| {
            anyhow::anyhow!("Supergraph '{}' not found in supergraphs/config.toml", name)
        })
    }

    pub fn get_gateway(&self, name: &str) -> Result<Arc<Gateway>> {
        self.gateways
            .iter()
            .find(|g| g.name() == name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Gateway '{}' not found in gateways/config.toml", name))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioConfig {
    pub supergraph: String,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupergraphConfig {
    pub subgraphs: Vec<String>,
}

fn load_scenarios(current_dir: &Path) -> Result<BTreeMap<String, ScenarioConfig>> {
    let config_path = current_dir.join("scenarios").join("config.toml");
    let content =
        std::fs::read_to_string(&config_path).context("Could not read scenarios/config.toml")?;
    toml::from_str(&content).context("Could not parse scenarios/config.toml")
}

fn load_supergraphs(current_dir: &Path) -> Result<BTreeMap<String, SupergraphConfig>> {
    let config_path = current_dir.join("supergraphs").join("config.toml");
    let content =
        std::fs::read_to_string(&config_path).context("Could not read supergraphs/config.toml")?;
    toml::from_str(&content).context("Could not parse supergraphs/config.toml")
}
