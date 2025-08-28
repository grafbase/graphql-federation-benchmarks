use anyhow::Result;
use duct::cmd;
use std::path::Path;

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

pub fn compose_down(path: &Path) -> Result<()> {
    tracing::debug!("Stopping subgraphs with docker compose at {:?}", path);

    cmd!("docker", "compose", "down", "-v")
        .dir(path)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to stop subgraphs: {}", e))?;

    tracing::debug!("Subgraphs stopped successfully");
    Ok(())
}
