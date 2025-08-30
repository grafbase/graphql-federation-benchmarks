use crate::benchmark::BenchmarkResult;
use crate::system::SystemInfo;
use std::collections::BTreeMap;

pub fn generate_report(
    timestamp: time::OffsetDateTime,
    results: &[BenchmarkResult],
    system_info: &SystemInfo,
) -> String {
    let mut grouped_results: BTreeMap<String, Vec<&BenchmarkResult>> = BTreeMap::new();

    for result in results {
        grouped_results
            .entry(result.benchmark.clone())
            .or_default()
            .push(result);
    }

    let mut report = String::new();

    // Add system information at the beginning
    report.push_str("# System Information\n\n");
    report.push_str(&format!("Date: {}\n", timestamp.date()));
    report.push_str(&format!("CPU: {}\n", system_info.cpu_model));
    report.push_str(&format!(
        "Memory: {:.1} GiB\n",
        system_info.total_memory_mib as f64 / 1024.0,
    ));
    if let Some(boost) = system_info.cpu_boost_enabled {
        report.push_str(&format!(
            "CPU Boost: {}\n",
            if boost { "Enabled" } else { "Disabled" }
        ));
    }
    if let Some(git_commit) = &system_info.git_commit {
        report.push_str(&format!("Git Commit: {}\n", git_commit));
    }
    if let Some(linux_version) = &system_info.linux_version {
        report.push_str(&format!("Linux Version: {}\n", linux_version));
    }
    if let Some(docker_version) = &system_info.docker_version {
        report.push_str(&format!("Docker Version: {}\n", docker_version));
    }
    report.push_str("\n# Benchmarks\n\n");

    for (benchmark_name, benchmark_results) in grouped_results {
        report.push_str(&format!("## {}\n\n", benchmark_name));

        // Calculate column widths for proper alignment
        let gateway_width = benchmark_results
            .iter()
            .map(|r| r.gateway.len())
            .max()
            .unwrap_or(7)
            .max(7); // At least as wide as "Gateway"

        // Requests table
        report.push_str("### Requests\n\n");
        report.push_str(&format!(
            "| {:<width$} | {:>8} | {:>8} | {:>25} |\n",
            "Gateway",
            "Requests",
            "Failures",
            "Subgraph requests (total)",
            width = gateway_width
        ));

        report.push_str(&format!(
            "| {:-<width$} | {:->8} | {:->8} | {:->25} |\n",
            ":",
            ":",
            ":",
            ":",
            width = gateway_width
        ));

        for result in benchmark_results.iter() {
            let http_req_duration = &result.k6_run.summary.metrics.http_req_duration.values;
            let failures = result.k6_run.summary.metrics.checks.values.fails;

            let sub = format!(
                "{} ({})",
                result.k6_run.summary.subgraph_stats.count / http_req_duration.count,
                result.k6_run.summary.subgraph_stats.count,
            );

            report.push_str(&format!(
                "| {:<width$} | {:>8} | {:>8} | {:>25} |\n",
                result.gateway,
                http_req_duration.count,
                failures,
                sub,
                width = gateway_width
            ));
        }

        report.push_str("\n### Latencies (ms)\n\n");

        // Latencies table
        report.push_str(&format!(
            "| {:<width$} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} |\n",
            "Gateway",
            "Min",
            "Med",
            "P90",
            "P95",
            "P99",
            "Max",
            width = gateway_width
        ));

        report.push_str(&format!(
            "| {:-<width$} | {:->7} | {:->7} | {:->7} | {:->7} | {:->7} | {:->7} |\n",
            ":",
            ":",
            ":",
            ":",
            ":",
            ":",
            ":",
            width = gateway_width
        ));

        for result in benchmark_results.iter() {
            let http_req_duration = &result.k6_run.summary.metrics.http_req_duration.values;

            report.push_str(&format!(
                "| {:<width$} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} |\n",
                result.gateway,
                http_req_duration.min,
                http_req_duration.med,
                http_req_duration.p90,
                http_req_duration.p95,
                http_req_duration.p99,
                http_req_duration.max,
                width = gateway_width
            ));
        }

        report.push_str("\n### Resources\n\n");

        // Resource usage table
        report.push_str(&format!(
            "| {:<width$} | {:>8} | {:>8} | {:>9} | {:>9} |\n",
            "Gateway",
            "CPU avg",
            "CPU max",
            "MEM avg",
            "MEM max",
            width = gateway_width
        ));

        report.push_str(&format!(
            "| {:-<width$} | {:->8} | {:->8} | {:->9} | {:->9} |\n",
            ":",
            ":",
            ":",
            ":",
            ":",
            width = gateway_width
        ));

        for result in benchmark_results {
            tracing::debug!(
                "Benchmark results: {}",
                serde_json::to_string_pretty(result).unwrap()
            );
            let resource_stats = &result.resource_stats;

            // u00A0 is a non-breaking space to prevent line breaks in the table
            report.push_str(&format!(
                "| {:<width$} | {:>7.0}% | {:>7.0}% | {:>5.0}\u{00A0}MiB | {:>5.0}\u{00A0}MiB |\n",
                result.gateway,
                resource_stats.cpu_usage_avg * 100.0,
                resource_stats.cpu_usage_max * 100.0,
                resource_stats.memory_mib_avg,
                resource_stats.memory_mib_max,
                width = gateway_width
            ));
        }

        report.push('\n');
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::BenchmarkResult;
    use crate::k6::{
        CheckMetric, CounterMetric, CounterValues, HttpReqFailedValues, K6Run, K6Summary,
        K6SummaryMetrics, SubgraphStats, TrendMetric, TrendValues,
    };
    use crate::resources::ResourceStats;
    use crate::system::SystemInfo;
    use std::time::Duration;

    #[test]
    fn test_generate_report_formatting() {
        let results = vec![
            BenchmarkResult {
                benchmark: "simple-query".to_string(),
                gateway: "Grafbase".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            data_received: CounterMetric {
                                values: CounterValues {
                                    count: 1000.0,
                                    rate: 100.0,
                                },
                            },
                            data_sent: CounterMetric {
                                values: CounterValues {
                                    count: 500.0,
                                    rate: 50.0,
                                },
                            },
                            http_req_duration: TrendMetric {
                                values: TrendValues {
                                    count: 251,
                                    avg: 20.5,
                                    min: 16.527396,
                                    med: 19.080906,
                                    max: 63.573805,
                                    p90: 21.212028,
                                    p95: 24.4168305,
                                    p99: 27.273214,
                                },
                            },
                            checks: CheckMetric {
                                values: HttpReqFailedValues { fails: 0 },
                            },
                            http_reqs: CounterMetric {
                                values: CounterValues {
                                    count: 251.0,
                                    rate: 50.03,
                                },
                            },
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.032, // 3.2%
                    cpu_usage_max: 0.105, // 10.5%
                    memory_mib_avg: 191.7,
                    memory_mib_max: 205.3,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
            BenchmarkResult {
                benchmark: "simple-query".to_string(),
                gateway: "Apollo Router".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            data_received: CounterMetric {
                                values: CounterValues {
                                    count: 1100.0,
                                    rate: 110.0,
                                },
                            },
                            data_sent: CounterMetric {
                                values: CounterValues {
                                    count: 550.0,
                                    rate: 55.0,
                                },
                            },
                            http_req_duration: TrendMetric {
                                values: TrendValues {
                                    count: 234,
                                    avg: 23.5,
                                    min: 18.234567,
                                    med: 21.543210,
                                    max: 72.345678,
                                    p90: 24.123456,
                                    p95: 27.234567,
                                    p99: 31.456789,
                                },
                            },
                            checks: CheckMetric {
                                values: HttpReqFailedValues { fails: 0 },
                            },
                            http_reqs: CounterMetric {
                                values: CounterValues {
                                    count: 250.0,
                                    rate: 49.8,
                                },
                            },
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.045, // 4.5%
                    cpu_usage_max: 0.152, // 15.2%
                    memory_mib_avg: 220.5,
                    memory_mib_max: 245.8,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
            BenchmarkResult {
                benchmark: "complex-nested-query".to_string(),
                gateway: "Grafbase".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            data_received: CounterMetric {
                                values: CounterValues {
                                    count: 2000.0,
                                    rate: 200.0,
                                },
                            },
                            data_sent: CounterMetric {
                                values: CounterValues {
                                    count: 1000.0,
                                    rate: 100.0,
                                },
                            },
                            http_req_duration: TrendMetric {
                                values: TrendValues {
                                    count: 234,
                                    avg: 45.5,
                                    min: 35.123456,
                                    med: 42.234567,
                                    max: 125.678901,
                                    p90: 55.345678,
                                    p95: 65.456789,
                                    p99: 85.567890,
                                },
                            },
                            checks: CheckMetric {
                                values: HttpReqFailedValues { fails: 1 },
                            },
                            http_reqs: CounterMetric {
                                values: CounterValues {
                                    count: 200.0,
                                    rate: 40.0,
                                },
                            },
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.125, // 12.5%
                    cpu_usage_max: 0.456, // 45.6%
                    memory_mib_avg: 512.3,
                    memory_mib_max: 1024.7,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
        ];

        let system_info = SystemInfo {
            cpu_model: "Test CPU Model".to_string(),
            total_memory_mib: 16384,
            cpu_boost_enabled: Some(true),
            git_commit: Some("abc123def456".to_string()),
            linux_version: Some("6.16.1".to_string()),
            docker_version: Some("24.0.7".to_string()),
        };
        let report = generate_report(
            time::macros::datetime!(2019-01-01 0:00 UTC),
            &results,
            &system_info,
        );
        insta::assert_snapshot!(report, @r"
        # System Information

        Date: 2019-01-01
        CPU: Test CPU Model
        Memory: 16.0 GiB
        CPU Boost: Enabled
        Git Commit: abc123def456
        Linux Version: 6.16.1
        Docker Version: 24.0.7

        # Benchmarks

        ## complex-nested-query

        ### Requests

        | Gateway  | Requests | Failures | Subgraph requests (total) |
        | :------- | -------: | -------: | ------------------------: |
        | Grafbase |      234 |        1 |                   2 (502) |

        ### Latencies (ms)

        | Gateway  |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :------- | ------: | ------: | ------: | ------: | ------: | ------: |
        | Grafbase |    35.1 |    42.2 |    55.3 |    65.5 |    85.6 |   125.7 |

        ### Resources

        | Gateway  |  CPU avg |  CPU max |   MEM avg |   MEM max |
        | :------- | -------: | -------: | --------: | --------: |
        | Grafbase |      12% |      46% |   512 MiB |  1025 MiB |

        ## simple-query

        ### Requests

        | Gateway       | Requests | Failures | Subgraph requests (total) |
        | :------------ | -------: | -------: | ------------------------: |
        | Grafbase      |      251 |        0 |                   2 (502) |
        | Apollo Router |      234 |        0 |                   2 (502) |

        ### Latencies (ms)

        | Gateway       |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :------------ | ------: | ------: | ------: | ------: | ------: | ------: |
        | Grafbase      |    16.5 |    19.1 |    21.2 |    24.4 |    27.3 |    63.6 |
        | Apollo Router |    18.2 |    21.5 |    24.1 |    27.2 |    31.5 |    72.3 |

        ### Resources

        | Gateway       |  CPU avg |  CPU max |   MEM avg |   MEM max |
        | :------------ | -------: | -------: | --------: | --------: |
        | Grafbase      |       3% |      10% |   192 MiB |   205 MiB |
        | Apollo Router |       4% |      15% |   220 MiB |   246 MiB |
        ");
    }
}
