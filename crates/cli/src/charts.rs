use crate::benchmark::BenchmarkResult;
use plotters::prelude::*;

const CHART_WIDTH: u32 = 800;
const CHART_HEIGHT: u32 = 600;

const GATEWAY_COLORS: &[RGBColor] = &[
    RGBColor(37, 99, 235),  // blue
    RGBColor(220, 38, 38),  // red
    RGBColor(22, 163, 74),  // green
    RGBColor(147, 51, 234), // purple
    RGBColor(245, 158, 11), // amber
    RGBColor(236, 72, 153), // pink
    RGBColor(20, 184, 166), // teal
    RGBColor(251, 146, 60), // orange
];

pub fn generate_latency_chart(
    scenario_name: &str,
    results: &[&BenchmarkResult],
) -> anyhow::Result<String> {
    // Collect data for valid results (no failures)
    let mut chart_data: Vec<(&str, f64, f64, f64)> = Vec::new();
    let mut max_latency = 0.0f64;

    for result in results {
        let failures = result
            .k6_run
            .summary
            .metrics
            .checks
            .as_ref()
            .map(|c| c.values.fails)
            .unwrap_or(0);

        if failures == 0
            && let Some(http_req_duration) = &result.k6_run.summary.metrics.http_req_duration
        {
            let values = &http_req_duration.values;
            chart_data.push((&result.gateway, values.med, values.p95, values.p99));
            max_latency = max_latency.max(values.p99);
        }
    }

    if chart_data.is_empty() {
        // Return empty SVG
        return Ok(String::from("<svg></svg>"));
    }

    let mut svg_buffer = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_buffer, (CHART_WIDTH, CHART_HEIGHT))
            .into_drawing_area();
        root.fill(&WHITE)?;

        // Add 10% margin to max value for better visualization
        let y_max = (max_latency * 1.1).ceil();

        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("{} - Latency Distribution", scenario_name),
                ("sans-serif", 30).into_font(),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f64..10f64, // X range for positioning bars
                0f64..y_max,
            )?;

        chart
            .configure_mesh()
            .y_desc("Latency (ms)")
            .x_desc("Percentiles")
            .x_label_formatter(&|x| match x.round() as i32 {
                1 => "Median".to_string(),
                5 => "P95".to_string(),
                9 => "P99".to_string(),
                _ => "".to_string(),
            })
            .draw()?;

        // Calculate bar width and positions
        let num_gateways = chart_data.len();
        let bar_width = 0.8 / num_gateways as f64;
        let group_positions = [1.0, 5.0, 9.0]; // X positions for Median, P95, P99

        // Draw bars for each gateway
        for (gateway_idx, (gateway_name, median, p95, p99)) in chart_data.iter().enumerate() {
            let color = GATEWAY_COLORS[gateway_idx % GATEWAY_COLORS.len()];
            let offset = (gateway_idx as f64 - num_gateways as f64 / 2.0 + 0.5) * bar_width;

            // Draw median bar
            let x = group_positions[0] + offset;
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x - bar_width / 2.0, 0.0), (x + bar_width / 2.0, *median)],
                color.filled(),
            )))?;

            // Draw p95 bar
            let x = group_positions[1] + offset;
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x - bar_width / 2.0, 0.0), (x + bar_width / 2.0, *p95)],
                color.filled(),
            )))?;

            // Draw p99 bar with legend
            let x = group_positions[2] + offset;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x - bar_width / 2.0, 0.0), (x + bar_width / 2.0, *p99)],
                    color.filled(),
                )))?
                .label(*gateway_name)
                .legend(move |(x, y)| Rectangle::new([(x, y), (x + 10, y + 10)], color.filled()));
        }

        // Draw legend
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
    }

    Ok(svg_buffer)
}

pub fn generate_latency_chart_to_file(
    scenario_name: &str,
    results: &[&BenchmarkResult],
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let svg_content = generate_latency_chart(scenario_name, results)?;
    std::fs::write(output_path, svg_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k6::{
        CheckMetric, HttpReqFailedValues, K6Run, K6Summary, K6SummaryMetrics, K6SummaryState,
        SubgraphStats, TrendMetric, TrendValues,
    };
    use crate::resources::ResourceStats;
    use std::time::Duration;

    #[test]
    fn test_generate_latency_chart() {
        let results = vec![
            BenchmarkResult {
                benchmark: "test-scenario".to_string(),
                gateway: "Gateway A".to_string(),
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
                benchmark: "test-scenario".to_string(),
                gateway: "Gateway B".to_string(),
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
        ];

        let refs: Vec<&BenchmarkResult> = results.iter().collect();
        let svg = generate_latency_chart("Test Scenario", &refs).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Scenario - Latency Distribution"));
        assert!(svg.contains("Gateway A"));
        assert!(svg.contains("Gateway B"));
    }
}

