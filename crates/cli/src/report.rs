use crate::benchmark::BenchmarkResult;
use crate::config::Config;
use crate::system::SystemInfo;
use std::collections::BTreeMap;

const ERR_PLACEHOLDER: &str = "errors";

pub struct ReportOptions {
    pub is_tty: bool,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self { is_tty: true }
    }
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
            .entry(result.scenario.clone())
            .or_default()
            .push(result);
    }

    let mut report = String::new();

    // Add system information at the beginning
    if options.is_tty {
        // In TTY mode, only show CPU boost status if available
        if let Some(boost) = system_info.cpu_boost_enabled {
            report.push_str(&format!(
                "CPU Boost: {}\n\n",
                if boost { "Enabled" } else { "Disabled" }
            ));
        }
    } else {
        // In file mode, show full system information
        report.push_str("# System Information\n\n");
        report.push_str(&format!("- Date: {}\n", timestamp.date()));
        report.push_str(&format!("- CPU: {}\n", system_info.cpu_model));
        report.push_str(&format!(
            "- Memory: {:.1} GiB\n",
            system_info.total_memory_mib as f64 / 1024.0,
        ));
        if let Some(boost) = system_info.cpu_boost_enabled {
            report.push_str(&format!(
                "- CPU Boost: {}\n",
                if boost { "Enabled" } else { "Disabled" }
            ));
        }
        if let Some(git_commit) = &system_info.git_commit {
            report.push_str(&format!("- Git Commit: {}\n", git_commit));
        }
        if let Some(linux_version) = &system_info.linux_version {
            report.push_str(&format!("- Linux Version: {}\n", linux_version));
        }
        if let Some(docker_version) = &system_info.docker_version {
            report.push_str(&format!("- Docker Version: {}\n", docker_version));
        }

        report.push_str("\n# Gateways\n\n");

        // Collect unique gateways from results
        let mut gateway_images = BTreeMap::new();

        for result in results {
            gateway_images
                .entry(&result.gateway.name)
                .or_insert_with(|| &result.gateway.config.image);
        }

        report.push_str("The following gateways were tested (as configured in `config.toml`):\n\n");
        for (name, image) in gateway_images {
            report.push_str(&format!("- {name}: {image}\n"));
        }
        report.push('\n');
    }

    for (scenario_name, benchmark_results) in grouped_results {
        report.push_str(&format!("# {}\n\n", scenario_name));

        // Add description if available from the config and not in tty mode
        let scenario = config.get_scenario(&scenario_name)?;
        if !options.is_tty && !scenario.description.is_empty() {
            report.push_str(&format!("{}\n\n", scenario.description));
        }

        // Calculate column widths for proper alignment
        let gateway_width = benchmark_results
            .iter()
            .map(|r| r.gateway.label().len())
            .max()
            .unwrap_or(7)
            .max(7); // At least as wide as "Gateway"

        // Latencies table first
        if !options.is_tty {
            report.push_str("## Latencies (ms)\n\n");
            // Add latency chart image before the table
            let latency_chart_path = format!("{}-latency.svg", scenario_name);
            report.push_str(&format!(
                "![Latency Chart](charts/{})\n\n",
                latency_chart_path
            ));
        }

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

        // Sort results for latencies table: by median (lowest first), errors at end
        let mut sorted_results = benchmark_results.clone();
        sorted_results.sort_by(|a, b| {
            match (a.is_valid(), b.is_valid()) {
                (true, true) => a.median_latency().total_cmp(&b.median_latency()),
                (true, false) => std::cmp::Ordering::Less, // a has data, b doesn't -> a comes first
                (false, true) => std::cmp::Ordering::Greater, // b has data, a doesn't -> b comes first
                (false, false) => a.gateway.label().cmp(b.gateway.label()), // Neither has data, sort by name
            }
        });

        for result in sorted_results.iter() {
            if result.has_failures() {
                report.push_str(&format!(
                    "| {:<width$} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} | {:>7} |\n",
                    result.gateway.label(),
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
                    result.gateway.label(),
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
                    result.gateway.label(),
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

        if !options.is_tty {
            report.push_str("\n## Resources\n\n");
            // Add efficiency chart image before the table
            let efficiency_chart_path = format!("{}-efficiency.svg", scenario_name);
            report.push_str(&format!(
                "![Efficiency Chart](charts/{})\n\n",
                efficiency_chart_path
            ));
        } else {
            report.push('\n');
        }

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

        // Sort results for resources table: by requests per core (highest first), errors at end
        let mut sorted_results = benchmark_results.clone();
        sorted_results.sort_by(|a, b| {
            match (a.is_valid(), b.is_valid()) {
                (true, true) => {
                    // Both have data, sort by requests per core (highest first)
                    let a_rpc = a.requests_per_core_s();
                    let b_rpc = b.requests_per_core_s();
                    b_rpc.total_cmp(&a_rpc) // Reversed for descending order
                }
                (true, false) => std::cmp::Ordering::Less, // a has data, b doesn't -> a comes first
                (false, true) => std::cmp::Ordering::Greater, // b has data, a doesn't -> b comes first
                (false, false) => a.gateway.label().cmp(b.gateway.label()), // Neither has data, sort by name
            }
        });

        for result in &sorted_results {
            let resource_stats = &result.resource_stats;

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

            if result.has_failures() {
                report.push_str(&format!(
                    "| {:<width$} | {:>12} | {:>7.0}% | {:>14} | {:>5.0}\u{00A0}MiB | {:>16} | {:>14} |\n",
                    result.gateway.label(),
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
                    result.gateway.label(),
                    cpu_str,
                    resource_stats.cpu_usage_max * 100.0,
                    mem_str,
                    resource_stats.memory_mib_max,
                    result.requests_per_core_s(),
                    result.requests_per_gb_s(),
                    width = gateway_width
                ));
            }
        }

        // Requests table last (after Resources)
        if !options.is_tty {
            report.push_str("\n## Requests\n\n");
            // Add quality chart image before the table
            let quality_chart_path = format!("{}-quality.svg", scenario_name);
            report.push_str(&format!(
                "![Quality Chart](charts/{})\n\n",
                quality_chart_path
            ));
        } else {
            report.push('\n');
        }

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

        // Sort results for requests table: by average subgraph requests (lowest first), errors at end
        let mut sorted_results = benchmark_results.clone();
        sorted_results.sort_by(|a, b| {
            match (a.is_valid(), b.is_valid()) {
                (true, true) => {
                    // Both have data, sort by average subgraph requests (lowest first)
                    let a_avg = a.average_subgraph_requests();
                    let b_avg = b.average_subgraph_requests();
                    a_avg.partial_cmp(&b_avg).unwrap()
                }
                (true, false) => std::cmp::Ordering::Less, // a has data, b doesn't -> a comes first
                (false, true) => std::cmp::Ordering::Greater, // b has data, a doesn't -> b comes first
                (false, false) => a.gateway.label().cmp(b.gateway.label()), // Neither has data, sort by name
            }
        });

        for result in sorted_results.iter() {
            let requests_count = result.request_count();
            let failures = result.failure_count();

            let decimal_places = match result.average_subgraph_requests() {
                x if x >= 100.0 => 0,
                x if x >= 10.0 => 1,
                _ => 2,
            };
            let sub = if requests_count > 0 {
                format!(
                    "{:.prec$} ({})",
                    result.average_subgraph_requests(),
                    result.subgraph_request_count(),
                    prec = decimal_places
                )
            } else {
                "0 (0)".to_string()
            };

            report.push_str(&format!(
                "| {:<width$} | {:>8} | {:>8} | {:>25} |\n",
                result.gateway.label(),
                requests_count,
                failures,
                sub,
                width = gateway_width
            ));
        }

        report.push('\n');
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::BenchmarkResult;
    use crate::config::{Config, Gateway, ScenarioConfig};
    use crate::k6::{
        CheckMetric, CounterMetric, CounterValues, HttpReqFailedValues, K6Run, K6Summary,
        K6SummaryMetrics, K6SummaryState, SubgraphStats, TrendMetric, TrendValues,
    };
    use crate::resources::ResourceStats;
    use crate::system::SystemInfo;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_generate_report_formatting() {
        // Create mock gateways
        let gateways = vec![
            Arc::new(Gateway {
                name: "a".to_string(),
                gateways_path: std::path::PathBuf::from("/test/gateways"),
                config: crate::config::GatewayConfig {
                    label: "Gateway A".to_string(),
                    image: "gateway-a:latest".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                },
            }),
            Arc::new(Gateway {
                name: "b".to_string(),
                gateways_path: std::path::PathBuf::from("/test/gateways"),
                config: crate::config::GatewayConfig {
                    label: "Gateway B".to_string(),
                    image: "gateway-b:v2.0".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                },
            }),
            Arc::new(Gateway {
                name: "c".to_string(),
                gateways_path: std::path::PathBuf::from("/test/gateways"),
                config: crate::config::GatewayConfig {
                    label: "Gateway C".to_string(),
                    image: "gateway-c:experimental".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                },
            }),
            Arc::new(Gateway {
                name: "d-noresponse".to_string(),
                gateways_path: std::path::PathBuf::from("/test/gateways"),
                config: crate::config::GatewayConfig {
                    label: "Gateway D".to_string(),
                    image: "gateway-d:broken".to_string(),
                    args: vec![],
                    env: HashMap::new(),
                },
            }),
        ];

        let results = vec![
            BenchmarkResult {
                scenario: "simple-query".to_string(),
                gateway: gateways[0].clone(),
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
                scenario: "simple-query".to_string(),
                gateway: gateways[1].clone(),
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
                scenario: "complex-nested-query".to_string(),
                gateway: gateways[2].clone(),
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
                scenario: "complex-nested-query".to_string(),
                gateway: gateways[3].clone(),
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
            gateways,
            current_dir: std::path::PathBuf::from("/test"),
        };

        // Use file mode (non-TTY) for test to get full output
        let report = generate_report_with_options(
            time::macros::datetime!(2019-01-01 0:00 UTC),
            &results,
            &system_info,
            &config,
            &ReportOptions { is_tty: false },
        )
        .unwrap();
        insta::assert_snapshot!(report, @r"
        # System Information

        - Date: 2019-01-01
        - CPU: Test CPU Model
        - Memory: 16.0 GiB
        - CPU Boost: Enabled
        - Git Commit: abc123def456
        - Linux Version: 6.16.1
        - Docker Version: 24.0.7

        # Gateways

        The following gateways were tested (as configured in `config.toml`):

        - **Gateway A**: `gateway-a:latest`
        - **Gateway B**: `gateway-b:v2.0`
        - **Gateway C**: `gateway-c:experimental`
        - **Gateway D**: `gateway-d:broken`

        # complex-nested-query

        Test scenario for complex nested GraphQL queries

        ## Latencies (ms)

        ![Latency Chart](charts/complex-nested-query-latency.svg)

        | Gateway      |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :----------- | ------: | ------: | ------: | ------: | ------: | ------: |
        | c            |  errors |  errors |  errors |  errors |  errors |  errors |
        | d-noresponse |    >60s |    >60s |    >60s |    >60s |    >60s |    >60s |

        ## Resources

        ![Efficiency Chart](charts/complex-nested-query-efficiency.svg)

        | Gateway      |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
        | :----------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
        | c            |      12% ±9% |      46% |   512 ±156 MiB |  1025 MiB |           errors |         errors |
        | d-noresponse |       1% ±0% |       2% |     100 ±5 MiB |   110 MiB |              0.0 |            0.0 |

        ## Requests

        ![Quality Chart](charts/complex-nested-query-quality.svg)

        | Gateway      | Requests | Failures | Subgraph requests (total) |
        | :----------- | -------: | -------: | ------------------------: |
        | c            |      234 |       10 |                2.15 (502) |
        | d-noresponse |        0 |        0 |                     0 (0) |

        # simple-query

        Test scenario for simple GraphQL queries

        ## Latencies (ms)

        ![Latency Chart](charts/simple-query-latency.svg)

        | Gateway |     Min |     Med |     P90 |     P95 |     P99 |     Max |
        | :------ | ------: | ------: | ------: | ------: | ------: | ------: |
        | a       |    16.5 |    19.1 |    21.2 |    24.4 |    27.3 |    63.6 |
        | b       |    18.2 |    21.5 |    24.1 |    27.2 |    31.5 |    72.3 |

        ## Resources

        ![Efficiency Chart](charts/simple-query-efficiency.svg)

        | Gateway |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
        | :------ | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
        | a       |       3% ±2% |      10% |     192 ±8 MiB |   205 MiB |            476.5 |          249.5 |
        | b       |       4% ±3% |      15% |    220 ±12 MiB |   246 MiB |            327.6 |          207.5 |

        ## Requests

        ![Quality Chart](charts/simple-query-quality.svg)

        | Gateway | Requests | Failures | Subgraph requests (total) |
        | :------ | -------: | -------: | ------------------------: |
        | a       |      251 |        0 |                2.00 (502) |
        | b       |      234 |        0 |                2.15 (502) |
        ");
    }
}
