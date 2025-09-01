use anyhow::Result;

pub async fn wait_for_gateway_health_with_logs(container_id: &str) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    const WAIT_DURATION_S: u64 = 30;
    let client = reqwest::Client::new();
    let health_query = r#"{"query":"{ __typename }"}"#;
    let expected_response = r#"{"data":{"__typename":"Query"}}"#;

    tracing::info!("Waiting for gateway to be healthy...");

    // Start log streaming process
    let mut log_process = Command::new("docker")
        .args(["logs", "-f", container_id])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Create readers for stdout and stderr
    let stdout = log_process.stdout.take().expect("Failed to get stdout");
    let stderr = log_process.stderr.take().expect("Failed to get stderr");
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Spawn tasks to read and print logs
    let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let log_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => println!("{}", line),
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => eprintln!("{}", line),
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                _ = log_rx.recv() => {
                    break;
                }
            }
        }
    });

    // Health check loop
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < WAIT_DURATION_S {
        if let Ok(response) = client
            .post("http://localhost:4000/graphql")
            .header("Content-Type", "application/json")
            .body(health_query)
            .send()
            .await
            && response.status().is_success()
        {
            let body = response.text().await?;
            if body.contains(expected_response) {
                tracing::info!("Gateway is healthy");

                // Stop log streaming
                let _ = log_tx.send(());
                let _ = log_process.kill().await;
                let _ = log_handle.await;

                return Ok(());
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    // Stop log streaming on timeout
    let _ = log_tx.send(());
    let _ = log_process.kill().await;
    let _ = log_handle.await;

    Err(anyhow::anyhow!(
        "Gateway did not become healthy after {} seconds",
        WAIT_DURATION_S
    ))
}