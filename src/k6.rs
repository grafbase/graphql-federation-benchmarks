use anyhow::Result;
use duct::cmd;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct K6Run {
    pub start: time::OffsetDateTime,
    pub end: time::OffsetDateTime,
    pub summary: K6Summary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct K6Summary {
    pub metrics: K6SummaryMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct K6SummaryMetrics {
    pub data_received: CounterMetric,
    pub data_sent: CounterMetric,
    pub http_req_duration: TrendMetric,
    pub http_req_failed: HttpReqFailedMetric,
    pub http_reqs: CounterMetric,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CounterMetric {
    pub values: CounterValues,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CounterValues {
    pub count: f64,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendMetric {
    pub values: TrendValues,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrendValues {
    pub avg: f64,
    pub min: f64,
    pub med: f64,
    pub max: f64,
    #[serde(rename = "p(90)")]
    pub p90: f64,
    #[serde(rename = "p(95)")]
    pub p95: f64,
    #[serde(rename = "p(99)")]
    pub p99: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpReqFailedMetric {
    pub values: HttpReqFailedValues,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpReqFailedValues {
    pub fails: u64,
}

pub async fn run(path: &Path) -> Result<K6Run> {
    let summary_path = path.join("summary.json");

    // Clean up any existing summary file
    if summary_path.exists() {
        std::fs::remove_file(&summary_path)?;
    }

    tracing::info!("Starting K6 test for benchmark at {:?}", path);

    let start = time::OffsetDateTime::now_utc();

    // Run K6 with summary export
    let output = cmd!(
        "k6",
        "run",
        "--summary-trend-stats",
        "avg,min,med,max,p(90),p(95),p(99)",
        "k6.js"
    )
    .dir(path)
    .run();

    let end = time::OffsetDateTime::now_utc();

    if let Err(e) = output {
        return Err(anyhow::anyhow!("K6 test failed: {}", e));
    }

    tracing::info!("K6 test completed successfully");

    let content = std::fs::read_to_string(summary_path)?;
    let summary: K6Summary = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse K6 summary: {}", e))?;

    Ok(K6Run {
        summary,
        start,
        end,
    })
}
