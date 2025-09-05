use crate::benchmark::BenchmarkResult;
use crate::charts;
use crate::config::Config;
use crate::system::SystemInfo;
use std::collections::BTreeMap;
use std::path::PathBuf;

const ERR_PLACEHOLDER: &str = "<err>";

#[derive(Default)]
pub struct ReportOptions {
    pub generate_charts: bool,
    pub charts_dir: Option<PathBuf>,
}

#[cfg(test)]
pub fn generate_report(
    timestamp: time::OffsetDateTime,
    results: &[BenchmarkResult],
    system_info: &SystemInfo,
    config: &Config,
) -> anyhow::Result<String> {
    generate_report_with_options(
        timestamp,
        results,
        system_info,
        config,
        &ReportOptions::default(),
    )
}

pub fn generate_report_with_options(
    timestamp: time::OffsetDateTime,
    results: &[BenchmarkResult],
    system_info: &SystemInfo,
    config: &Config,
    options: &ReportOptions,
) -> anyhow::Result<String> {
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

        // Add description if available from the config
        let scenario = config.get_scenario(&benchmark_name)?;
        if !scenario.description.is_empty() {
            report.push_str(&format!("{}\n\n", scenario.description));
        }

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
            let requests_count = result
                .k6_run
                .summary
                .metrics
                .http_req_duration
                .as_ref()
                .map(|m| m.values.count)
                .unwrap_or(0);

            let failures = result
                .k6_run
                .summary
                .metrics
                .checks
                .as_ref()
                .map(|c| c.values.fails)
                .unwrap_or(0);

            let sub = if requests_count > 0 {
                format!(
                    "{:.2} ({})",
                    result.k6_run.summary.subgraph_stats.count as f64 / requests_count as f64,
                    result.k6_run.summary.subgraph_stats.count,
                )
            } else {
                "0 (0)".to_string()
            };

            report.push_str(&format!(
                "| {:<width$} | {:>8} | {:>8} | {:>25} |\n",
                result.gateway,
                requests_count,
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
            let failures = result
                .k6_run
                .summary
                .metrics
                .checks
                .as_ref()
                .map(|c| c.values.fails)
                .unwrap_or(0);

            if failures > 0 {
                report.push_str(&format!(
                    "| {:<width$} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} |\n",
                    result.gateway,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    width = gateway_width
                ));
            } else if let Some(http_req_duration) = &result.k6_run.summary.metrics.http_req_duration
            {
                let values = &http_req_duration.values;
                report.push_str(&format!(
                    "| {:<width$} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} | {:>7.1} |\n",
                    result.gateway,
                    values.min,
                    values.med,
                    values.p90,
                    values.p95,
                    values.p99,
                    values.max,
                    width = gateway_width
                ));
            } else {
                // No responses at all - show > test duration for all latencies
                let duration_s = result.k6_run.summary.state.test_run_duration_ms / 1000.0;
                let duration_str = format!(">{:.0}s", duration_s);
                report.push_str(&format!(
                    "| {:<width$} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} |\n",
                    result.gateway,
                    duration_str,
                    duration_str,
                    duration_str,
                    duration_str,
                    duration_str,
                    duration_str,
                    width = gateway_width
                ));
            }
        }

        // Generate and add chart if requested
        if options.generate_charts {
            report.push('\n');

            if let Some(charts_dir) = &options.charts_dir {
                // Save chart to file
                std::fs::create_dir_all(charts_dir)?;
                let chart_filename = format!("{}-latency.svg", benchmark_name.replace(' ', "-"));
                let chart_path = charts_dir.join(&chart_filename);

                charts::generate_latency_chart_to_file(
                    &benchmark_name,
                    &benchmark_results,
                    &chart_path,
                )?;

                report.push_str(&format!("![Latency Chart](./charts/{})\n", chart_filename));
            } else {
                // Embed chart as base64 data URL
                let svg_content =
                    charts::generate_latency_chart(&benchmark_name, &benchmark_results)?;
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&svg_content);
                report.push_str(&format!(
                    "![Latency Chart](data:image/svg+xml;base64,{})\n",
                    encoded
                ));
            }
            report.push('\n');
        }

        report.push_str("\n### Resources\n\n");

        // Resource usage table
        report.push_str(&format!(
            "| {:<width$} | {:>12} | {:>8} | {:>14} | {:>9} | {:>16} | {:>14} |\n",
            "Gateway",
            "CPU",
            "CPU max",
            "Memory",
            "MEM max",
            "requests/core.s",
            "requests/GB.s",
            width = gateway_width
        ));

        report.push_str(&format!(
            "| {:-<width$} | {:->12} | {:->8} | {:->14} | {:->9} | {:->16} | {:->14} |\n",
            ":",
            ":",
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

            let failures = result
                .k6_run
                .summary
                .metrics
                .checks
                .as_ref()
                .map(|c| c.values.fails)
                .unwrap_or(0);

            let resource_stats = &result.resource_stats;

            // Calculate request rate from K6 metrics
            let request_rate = result
                .k6_run
                .summary
                .metrics
                .http_reqs
                .as_ref()
                .map(|m| m.values.rate)
                .unwrap_or(0.0);

            // Calculate requests per CPU core second
            // cpu_usage_max is in cores (1.0 = 1 core, 2.0 = 2 cores, etc.)
            let requests_per_core_s = if resource_stats.cpu_usage_max > 0.0 {
                request_rate / resource_stats.cpu_usage_max
            } else {
                0.0
            };

            // Calculate requests per GB second
            // memory_mib_max is in MiB, convert to GB (1 GB = 1024 MiB)
            let memory_gb = resource_stats.memory_mib_max / 1024.0;
            let requests_per_gb_s = if memory_gb > 0.0 {
                request_rate / memory_gb
            } else {
                0.0
            };

            // Format CPU and Memory as mean ± std dev
            let cpu_str = format!(
                "{:.0}% ±{:.0}%",
                resource_stats.cpu_usage_avg * 100.0,
                resource_stats.cpu_usage_std * 100.0
            );
            let mem_str = format!(
                "{:.0} ±{:.0}\u{00A0}MiB",
                resource_stats.memory_mib_avg, resource_stats.memory_mib_std
            );

            if failures > 0 {
                report.push_str(&format!(
                    "| {:<width$} | {:>12} | {:>7.0}% | {:>14} | {:>5.0}\u{00A0}MiB | {:>16} | {:>14} |\n",
                    result.gateway,
                    cpu_str,
                    resource_stats.cpu_usage_max * 100.0,
                    mem_str,
                    resource_stats.memory_mib_max,
                    ERR_PLACEHOLDER,
                    ERR_PLACEHOLDER,
                    width = gateway_width
                ));
            } else {
                // u00A0 is a non-breaking space to prevent line breaks in the table
                report.push_str(&format!(
                    "| {:<width$} | {:>12} | {:>7.0}% | {:>14} | {:>5.0}\u{00A0}MiB | {:>16.1} | {:>14.1} |\n",
                    result.gateway,
                    cpu_str,
                    resource_stats.cpu_usage_max * 100.0,
                    mem_str,
                    resource_stats.memory_mib_max,
                    requests_per_core_s,
                    requests_per_gb_s,
                    width = gateway_width
                ));
            }
        }

        report.push('\n');
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::BenchmarkResult;
    use crate::config::{Config, ScenarioConfig};
    use crate::k6::{
        CheckMetric, CounterMetric, CounterValues, HttpReqFailedValues, K6Run, K6Summary,
        K6SummaryMetrics, K6SummaryState, SubgraphStats, TrendMetric, TrendValues,
    };
    use crate::resources::ResourceStats;
    use crate::system::SystemInfo;
    use std::collections::{BTreeMap, HashMap};
    use std::time::Duration;

    #[test]
    fn test_generate_report_formatting() {
        let results = vec![
            BenchmarkResult {
                benchmark: "simple-query".to_string(),
                gateway: "A".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        state: K6SummaryState {
                            test_run_duration_ms: 60000.0,
                        },
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            http_req_duration: Some(TrendMetric {
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
                            }),
                            checks: Some(CheckMetric {
                                values: HttpReqFailedValues { fails: 0 },
                            }),
                            http_reqs: Some(CounterMetric {
                                values: CounterValues {
                                    count: 251.0,
                                    rate: 50.03,
                                },
                            }),
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.032, // 3.2%
                    cpu_usage_max: 0.105, // 10.5%
                    cpu_usage_std: 0.015, // 1.5%
                    memory_mib_avg: 191.7,
                    memory_mib_max: 205.3,
                    memory_mib_std: 8.2,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
            BenchmarkResult {
                benchmark: "simple-query".to_string(),
                gateway: "B".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        state: K6SummaryState {
                            test_run_duration_ms: 60000.0,
                        },
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            http_req_duration: Some(TrendMetric {
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
                            }),
                            checks: Some(CheckMetric {
                                values: HttpReqFailedValues { fails: 0 },
                            }),
                            http_reqs: Some(CounterMetric {
                                values: CounterValues {
                                    count: 250.0,
                                    rate: 49.8,
                                },
                            }),
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.045, // 4.5%
                    cpu_usage_max: 0.152, // 15.2%
                    cpu_usage_std: 0.028, // 2.8%
                    memory_mib_avg: 220.5,
                    memory_mib_max: 245.8,
                    memory_mib_std: 12.5,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
            BenchmarkResult {
                benchmark: "complex-nested-query".to_string(),
                gateway: "C".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        state: K6SummaryState {
                            test_run_duration_ms: 60000.0,
                        },
                        subgraph_stats: SubgraphStats { count: 502 },
                        metrics: K6SummaryMetrics {
                            http_req_duration: Some(TrendMetric {
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
                            }),
                            checks: Some(CheckMetric {
                                values: HttpReqFailedValues { fails: 10 },
                            }),
                            http_reqs: Some(CounterMetric {
                                values: CounterValues {
                                    count: 200.0,
                                    rate: 40.0,
                                },
                            }),
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.125, // 12.5%
                    cpu_usage_max: 0.456, // 45.6%
                    cpu_usage_std: 0.087, // 8.7%
                    memory_mib_avg: 512.3,
                    memory_mib_max: 1024.7,
                    memory_mib_std: 156.4,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
            // Add test case for gateway with no responses
            BenchmarkResult {
                benchmark: "complex-nested-query".to_string(),
                gateway: "D-NoResponse".to_string(),
                k6_run: K6Run {
                    start: time::OffsetDateTime::now_utc(),
                    end: time::OffsetDateTime::now_utc(),
                    summary: K6Summary {
                        state: K6SummaryState {
                            test_run_duration_ms: 60000.0,
                        },
                        subgraph_stats: SubgraphStats { count: 0 },
                        metrics: K6SummaryMetrics {
                            http_req_duration: None,
                            checks: None,
                            http_reqs: None,
                        },
                    },
                },
                resource_stats: ResourceStats {
                    cpu_usage_avg: 0.01,
                    cpu_usage_max: 0.02,
                    cpu_usage_std: 0.005,
                    memory_mib_avg: 100.0,
                    memory_mib_max: 110.0,
                    memory_mib_std: 5.0,
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

        // Create a mock Config with scenario descriptions
        let mut scenarios = BTreeMap::new();
        scenarios.insert(
            "simple-query".to_string(),
            ScenarioConfig {
                supergraph: "test".to_string(),
                description: "Test scenario for simple GraphQL queries".to_string(),
                env: HashMap::new(),
            },
        );
        scenarios.insert(
            "complex-nested-query".to_string(),
            ScenarioConfig {
                supergraph: "test".to_string(),
                description: "Test scenario for complex nested GraphQL queries".to_string(),
                env: HashMap::new(),
            },
        );

        let config = Config {
            scenarios,
            supergraphs: BTreeMap::new(),
            gateways: Vec::new(),
            current_dir: std::path::PathBuf::from("/test"),
        };

        let report = generate_report(
            time::macros::datetime!(2019-01-01 0:00 UTC),
            &results,
            &system_info,
            &config,
        )
        .unwrap();
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

        Test scenario for complex nested GraphQL queries

        ### Requests

        | Gateway      | Requests | Failures | Subgraph requests (total) |
        | :----------- | -------: | -------: | ------------------------: |
        | C            |      234 |       10 |                2.15 (502) |
        | D-NoResponse |        0 |        0 |                     0 (0) |

        ### Latencies (ms)

        | Gateway      |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :----------- | ------: | ------: | ------: | ------: | ------: | ------: |
        | C            |   <err> |   <err> |   <err> |   <err> |   <err> |   <err> |
        | D-NoResponse |    >60s |    >60s |    >60s |    >60s |    >60s |    >60s |

        ### Resources

        | Gateway      |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
        | :----------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
        | C            |      12% ±9% |      46% |   512 ±156 MiB |  1025 MiB |            <err> |          <err> |
        | D-NoResponse |       1% ±0% |       2% |     100 ±5 MiB |   110 MiB |              0.0 |            0.0 |

        ## simple-query

        Test scenario for simple GraphQL queries

        ### Requests

        | Gateway | Requests | Failures | Subgraph requests (total) |
        | :------ | -------: | -------: | ------------------------: |
        | A       |      251 |        0 |                2.00 (502) |
        | B       |      234 |        0 |                2.15 (502) |

        ### Latencies (ms)

        | Gateway |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :------ | ------: | ------: | ------: | ------: | ------: | ------: |
        | A       |    16.5 |    19.1 |    21.2 |    24.4 |    27.3 |    63.6 |
        | B       |    18.2 |    21.5 |    24.1 |    27.2 |    31.5 |    72.3 |

        ### Resources

        | Gateway |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
        | :------ | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
        | A       |       3% ±2% |      10% |     192 ±8 MiB |   205 MiB |            476.5 |          249.5 |
        | B       |       4% ±3% |      15% |    220 ±12 MiB |   246 MiB |            327.6 |          207.5 |
        ");
    }
}
