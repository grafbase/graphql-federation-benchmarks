use anyhow::{Context as _, Result};
use fast_glob::glob_match;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::docker::{self, ContainerId};

/// The merged configuration file structure
#[derive(Debug, Deserialize)]
struct TomlConfig {
    scenarios: BTreeMap<String, ScenarioConfig>,
    supergraphs: BTreeMap<String, SupergraphConfig>,
    gateways: BTreeMap<String, GatewayConfig>,
}

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
        // Load the merged config file from root
        let config_path = current_dir.join("config.toml");
        let content = std::fs::read_to_string(&config_path)
            .context("Could not read config.toml from root directory")?;
        let merged_config: TomlConfig =
            toml::from_str(&content).context("Could not parse config.toml")?;

        // Convert gateways to the expected format
        let gateways = build_all(&current_dir, merged_config.gateways, None)?;

        Ok(Self {
            scenarios: merged_config.scenarios,
            supergraphs: merged_config.supergraphs,
            gateways,
            current_dir,
        })
    }

    pub fn get_scenario(&self, name: &str) -> Result<&ScenarioConfig> {
        self.scenarios
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Scenario '{}' not found in config.toml", name))
    }

    pub fn get_supergraph(&self, name: &str) -> Result<&SupergraphConfig> {
        self.supergraphs
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Supergraph '{}' not found in config.toml", name))
    }

    pub fn get_gateway(&self, name: &str) -> Result<Arc<Gateway>> {
        self.gateways
            .iter()
            .find(|g| g.name() == name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Gateway '{}' not found in config.toml", name))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioConfig {
    pub supergraph: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SupergraphConfig {
    pub subgraphs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayConfig {
    pub label: String,
    pub image: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub struct Gateway {
    pub name: String,
    pub gateways_path: PathBuf,
    pub config: GatewayConfig,
}

impl Gateway {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label(&self) -> &str {
        &self.config.label
    }

    pub fn start_with_supergraph(&self, supergraph_path: &Path) -> Result<ContainerId> {
        let volumes = vec![
            (
                self.gateways_path.to_string_lossy().to_string(),
                "/gateways".to_string(),
            ),
            (
                supergraph_path.to_string_lossy().to_string(),
                "/supergraph".to_string(),
            ),
        ];

        docker::run(
            &self.config.image,
            self.config.env.iter().map(|(k, v)| (k.clone(), v.clone())),
            volumes.into_iter(),
            self.config.args.clone().into_iter(),
        )
    }
}

/// Load gateways from the merged config structure
fn build_all(
    current_dir: &Path,
    gateways: BTreeMap<String, GatewayConfig>,
    filter: Option<String>,
) -> Result<Vec<Arc<Gateway>>> {
    let gateways_path = current_dir.join("gateways");

    Ok(gateways
        .into_iter()
        .filter(|(name, _)| {
            if let Some(ref filter) = filter {
                glob_match(filter, name)
            } else {
                true
            }
        })
        .map(|(name, config)| {
            Arc::new(Gateway {
                gateways_path: gateways_path.clone(),
                name: name.to_lowercase(),
                config,
            })
        })
        .collect())
}
