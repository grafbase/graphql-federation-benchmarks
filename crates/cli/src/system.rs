use anyhow::{Context as _, Result};
use serde::Serialize;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub cpu_model: String,
    pub total_memory_mib: u64,
    pub cpu_boost_enabled: Option<bool>,
    pub git_commit: Option<String>,
    pub linux_version: Option<String>,
    pub docker_version: Option<String>,
}

impl SystemInfo {
    pub fn detect() -> Result<Self> {
        let cpu_model = detect_cpu_model().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect CPU model: {}", e);
            "Unknown CPU".to_string()
        });

        let total_memory_mib = detect_total_memory_mib().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect total memory: {}", e);
            0
        });

        let cpu_boost_enabled = detect_cpu_boost().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect CPU boost status: {}", e);
            None
        });

        let git_commit = detect_git_commit().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect git commit: {}", e);
            None
        });

        let linux_version = detect_linux_version().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect Linux version: {}", e);
            None
        });

        let docker_version = detect_docker_version().unwrap_or_else(|e| {
            tracing::debug!("Failed to detect Docker version: {}", e);
            None
        });

        tracing::debug!(
            "System info: CPU={}, Memory={}MiB, Boost={:?}, Git={:?}, Linux={:?}, Docker={:?}",
            cpu_model,
            total_memory_mib,
            cpu_boost_enabled,
            git_commit,
            linux_version,
            docker_version
        );

        Ok(Self {
            cpu_model,
            total_memory_mib,
            cpu_boost_enabled,
            git_commit,
            linux_version,
            docker_version,
        })
    }
}

fn detect_cpu_model() -> Result<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;

    for line in cpuinfo.lines() {
        if line.starts_with("model name")
            && let Some(model) = line.split(':').nth(1)
        {
            return Ok(model.trim().to_string());
        }
    }

    Err(anyhow::anyhow!("CPU model not found in /proc/cpuinfo"))
}

fn detect_total_memory_mib() -> Result<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;

    for line in meminfo.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            let total = value
                .replace("kB", "")
                .trim()
                .parse::<u64>()
                .context("Parsing MemTotal value")?;
            return Ok(total >> 10);
        }
    }

    Err(anyhow::anyhow!("MemTotal not found in /proc/meminfo"))
}

fn detect_cpu_boost() -> Result<Option<bool>> {
    // Try AMD boost file first
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/cpufreq/boost") {
        return Ok(Some(content.trim() == "1"));
    }

    // Try Intel turbo boost file
    if let Ok(content) = fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        // Note: Intel's no_turbo is inverted (0 means turbo is enabled)
        return Ok(Some(content.trim() == "0"));
    }

    // CPU boost information not available
    Ok(None)
}

fn detect_git_commit() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to run git rev-parse")?;

    if output.status.success() {
        let commit = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();
        Ok(Some(commit))
    } else {
        Ok(None)
    }
}

fn detect_linux_version() -> Result<Option<String>> {
    // Try reading from /proc/version first
    if let Ok(version) = fs::read_to_string("/proc/version")
        && let Some(version_str) = version.split_whitespace().nth(2)
    {
        return Ok(Some(version_str.to_string()));
    }

    // Fallback to uname -r
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .context("Failed to run uname")?;

    if output.status.success() {
        let version = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in uname output")?
            .trim()
            .to_string();
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

fn detect_docker_version() -> Result<Option<String>> {
    let output = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output()
        .context("Failed to run docker version")?;

    if output.status.success() {
        let version = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in docker output")?
            .trim()
            .to_string();
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_detect() {
        // This test will pass on Linux systems with /proc filesystem
        let info = SystemInfo::detect();
        assert!(info.is_ok());

        let info = info.unwrap();
        // CPU model should not be empty on a real system
        assert!(!info.cpu_model.is_empty());
        // Memory should be greater than 0 on a real system
        assert!(info.total_memory_mib > 0);
    }
}
