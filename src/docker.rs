use anyhow::Result;
use duct::cmd;
use std::path::Path;

use crate::config::GatewayConfig;

pub fn compose_up(path: &Path) -> Result<()> {
    tracing::debug!("Starting subgraphs with docker compose at {:?}", path);

    cmd!(
        "docker",
        "compose",
        "up",
        "-d",
        "--wait",
        "--build",
        "--force-recreate"
    )
    .dir(path)
    .run()
    .map_err(|e| anyhow::anyhow!("Failed to start subgraphs: {}", e))?;

    tracing::debug!("Subgraphs started successfully");
    Ok(())
}

pub fn start_gateway(
    gateway_name: &str,
    config: &GatewayConfig,
    benchmark_path: &Path,
) -> Result<String> {
    let gateway_path = benchmark_path.join("gateways").join(gateway_name);

    if !gateway_path.exists() {
        return Err(anyhow::anyhow!(
            "Gateway directory not found: {:?}",
            gateway_path
        ));
    }

    cmd!("docker", "pull", &config.image)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to pull gateway image: {}", e))?;

    let mut args = vec![
        "run".to_string(),
        "-d".to_string(),
        "--rm".to_string(),
        "--network".to_string(),
        "host".to_string(),
        "-v".to_string(),
        format!("{}:/data", gateway_path.display()),
    ];

    for (key, value) in &config.environment {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }

    args.push(config.image.clone());
    args.extend(config.arguments.clone());

    tracing::debug!(
        "Starting gateway '{}': docker run {}",
        gateway_name,
        args.join(" ")
    );

    let container_id = cmd("docker", &args)
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to start gateway container: {}", e))?;

    let container_id = container_id.trim().to_string();
    tracing::debug!("Gateway container started with ID: {}", container_id);

    Ok(container_id)
}

pub fn stop(container_id: &str) -> Result<()> {
    tracing::debug!("Stopping container: {}", container_id);

    cmd!("docker", "stop", container_id)
        .stdout_null()
        .stderr_null()
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to stop container: {}", e))?;

    tracing::debug!("Container stopped and removed");
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
