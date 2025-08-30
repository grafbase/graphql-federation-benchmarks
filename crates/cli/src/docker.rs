use anyhow::Result;
use duct::cmd;
use std::{collections::HashMap, path::Path};

pub fn compose_up(path: &Path, services: &[String], env: &HashMap<String, String>) -> Result<()> {
    if services.is_empty() {
        return Err(anyhow::anyhow!("No services specified to start"));
    }

    tracing::debug!(
        "Starting specific subgraphs {:?} with docker compose at {:?}",
        services,
        path
    );

    let mut args = vec![
        "compose".to_string(),
        "up".to_string(),
        "-d".to_string(),
        "--wait".to_string(),
        "--build".to_string(),
        "--force-recreate".to_string(),
    ];
    args.extend(services.iter().cloned());

    let mut docker_cmd = cmd("docker", &args);

    // Add environment variables
    for (key, value) in env {
        docker_cmd = docker_cmd.env(key, value);
        tracing::debug!("Setting environment variable: {}={}", key, value);
    }

    docker_cmd
        .dir(path)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to start subgraphs: {}", e))?;

    tracing::debug!("Subgraphs {:?} started successfully", services);
    Ok(())
}

pub fn compose_down(path: &Path) -> Result<()> {
    tracing::debug!("Stopping subgraphs with docker compose at {:?}", path);

    cmd!("docker", "compose", "down", "-v")
        .dir(path)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to stop subgraphs: {}", e))?;

    tracing::debug!("Subgraphs stopped successfully");
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ContainerId(String);

impl std::ops::Deref for ContainerId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn run(
    image: &str,
    env: impl Iterator<Item = (String, String)>,
    volumes: impl Iterator<Item = (String, String)>,
    arguments: impl Iterator<Item = String>,
) -> Result<ContainerId> {
    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--network".to_string(),
        "host".to_string(),
    ];

    for (host_dir, guest_dir) in volumes {
        args.push("-v".to_string());
        args.push(format!("{}:{}", host_dir, guest_dir));
    }

    for (key, value) in env {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(image.to_string());
    args.extend(arguments);

    tracing::debug!("docker {}", args.join(" "));

    let out = cmd("docker", &args)
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to start gateway container: {}", e))?;

    let id = out.lines().next().unwrap().trim().to_string();
    tracing::debug!("Gateway container started with ID: {}", id);

    Ok(ContainerId(id))
}

pub fn stop(container_id: &str) -> Result<()> {
    tracing::debug!("Stopping container: {}", container_id);

    let stop_result = cmd!("docker", "stop", "-t", "2", container_id)
        .stdout_null()
        .stderr_null()
        .run();

    cmd!("docker", "rm", container_id)
        .stdout_null()
        .stderr_null()
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to stop container: {}", e))?;

    let _ = stop_result?;

    tracing::debug!("Container stopped and removed");
    Ok(())
}
