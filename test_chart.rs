use cli::charts::generate_latency_chart_to_file;
use cli::benchmark::BenchmarkResult;
use cli::k6::{
    CheckMetric, HttpReqFailedValues, K6Run, K6Summary, K6SummaryMetrics, K6SummaryState,
    SubgraphStats, TrendMetric, TrendValues,
};
use cli::resources::ResourceStats;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let results = vec![
        BenchmarkResult {
            benchmark: "big-response".to_string(),
            gateway: "Grafbase".to_string(),
            k6_run: K6Run {
                start: time::OffsetDateTime::now_utc(),
                end: time::OffsetDateTime::now_utc(),
                summary: K6Summary {
                    state: K6SummaryState {
                        test_run_duration_ms: 60000.0,
                    },
                    subgraph_stats: SubgraphStats { count: 100 },
                    metrics: K6SummaryMetrics {
                        http_req_duration: Some(TrendMetric {
                            values: TrendValues {
                                count: 100,
                                avg: 25.0,
                                min: 10.0,
                                med: 20.0,
                                max: 100.0,
                                p90: 35.0,
                                p95: 45.0,
                                p99: 80.0,
                            },
                        }),
                        checks: Some(CheckMetric {
                            values: HttpReqFailedValues { fails: 0 },
                        }),
                        http_reqs: None,
                    },
                },
            },
            resource_stats: ResourceStats {
                cpu_usage_avg: 0.5,
                cpu_usage_max: 0.8,
                cpu_usage_std: 0.1,
                memory_mib_avg: 256.0,
                memory_mib_max: 512.0,
                memory_mib_std: 50.0,
                throttled_time: Duration::from_secs(0),
                count: 100,
            },
        },
        BenchmarkResult {
            benchmark: "big-response".to_string(),
            gateway: "Apollo Router".to_string(),
            k6_run: K6Run {
                start: time::OffsetDateTime::now_utc(),
                end: time::OffsetDateTime::now_utc(),
                summary: K6Summary {
                    state: K6SummaryState {
                        test_run_duration_ms: 60000.0,
                    },
                    subgraph_stats: SubgraphStats { count: 100 },
                    metrics: K6SummaryMetrics {
                        http_req_duration: Some(TrendMetric {
                            values: TrendValues {
                                count: 100,
                                avg: 30.0,
                                min: 15.0,
                                med: 25.0,
                                max: 120.0,
                                p90: 40.0,
                                p95: 55.0,
                                p99: 95.0,
                            },
                        }),
                        checks: Some(CheckMetric {
                            values: HttpReqFailedValues { fails: 0 },
                        }),
                        http_reqs: None,
                    },
                },
            },
            resource_stats: ResourceStats {
                cpu_usage_avg: 0.6,
                cpu_usage_max: 0.9,
                cpu_usage_std: 0.15,
                memory_mib_avg: 300.0,
                memory_mib_max: 600.0,
                memory_mib_std: 60.0,
                throttled_time: Duration::from_secs(0),
                count: 100,
            },
        },
        BenchmarkResult {
            benchmark: "big-response".to_string(),
            gateway: "Cosmo Router".to_string(),
            k6_run: K6Run {
                start: time::OffsetDateTime::now_utc(),
                end: time::OffsetDateTime::now_utc(),
                summary: K6Summary {
                    state: K6SummaryState {
                        test_run_duration_ms: 60000.0,
                    },
                    subgraph_stats: SubgraphStats { count: 100 },
                    metrics: K6SummaryMetrics {
                        http_req_duration: Some(TrendMetric {
                            values: TrendValues {
                                count: 100,
                                avg: 28.0,
                                min: 12.0,
                                med: 22.0,
                                max: 110.0,
                                p90: 38.0,
                                p95: 50.0,
                                p99: 88.0,
                            },
                        }),
                        checks: Some(CheckMetric {
                            values: HttpReqFailedValues { fails: 0 },
                        }),
                        http_reqs: None,
                    },
                },
            },
            resource_stats: ResourceStats {
                cpu_usage_avg: 0.55,
                cpu_usage_max: 0.85,
                cpu_usage_std: 0.12,
                memory_mib_avg: 280.0,
                memory_mib_max: 550.0,
                memory_mib_std: 55.0,
                throttled_time: Duration::from_secs(0),
                count: 100,
            },
        },
    ];

    let refs: Vec<&BenchmarkResult> = results.iter().collect();
    generate_latency_chart_to_file("Big Response Scenario", &refs, std::path::Path::new("latency_chart.svg"))?;
    
    println!("Chart generated: latency_chart.svg");
    
    Ok(())
}