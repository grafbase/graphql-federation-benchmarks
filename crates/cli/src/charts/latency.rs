use super::*;
use crate::benchmark::BenchmarkResult;
use plotters::prelude::*;

pub fn generate_latency_chart(
    scenario_name: &str,
    results: &[&BenchmarkResult],
) -> anyhow::Result<String> {
    use plotters::style::IntoFont;

    let mut buffer = String::new();
    {
        let root =
            SVGBackend::with_string(&mut buffer, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();

        // Chart background
        root.fill(&CHART_BACKGROUND)?;

        // Calculate dynamic legend width based on gateway names
        let legend_width = calculate_legend_width(results);

        // Split the drawing area: chart on the left, legend on the right
        let (chart_area, legend_area) = root.split_horizontally(CHART_WIDTH - legend_width);

        // Create sorted vector of (gateway_name, TrendValues), sorted by median (lowest first)
        // Exclude gateways with failures
        let mut gateway_data: Vec<(&str, &crate::k6::TrendValues)> = results
            .iter()
            .filter_map(|r| {
                // Only include if no failures and has duration data
                if r.is_valid() {
                    r.k6_run
                        .summary
                        .metrics
                        .http_req_duration
                        .as_ref()
                        .map(|metric| (r.gateway.as_str(), &metric.values))
                } else {
                    None
                }
            })
            .collect();

        gateway_data.sort_by(|a, b| a.1.med.partial_cmp(&b.1.med).unwrap());

        // Create a color mapping based on alphabetically sorted gateway names for consistency
        let color_map = create_color_map(results);

        // Find max latency for y-axis scaling
        let max_latency = gateway_data
            .iter()
            .flat_map(|(_, values)| vec![values.med, values.p95, values.p99])
            .fold(0.0f64, |acc, val| acc.max(val));

        let y_max = (max_latency * 1.1).ceil();

        let percentile_labels = ["Median", "p95", "p99"];

        // Draw bars for each gateway
        // We'll use a numeric x-axis for proper bar positioning
        let num_gateways = gateway_data.len();

        let mut chart = ChartBuilder::on(&chart_area)
            .caption(
                format!("{} - latencies", scenario_name),
                (FONT_FAMILY, TITLE_FONT_SIZE).into_font(),
            )
            .margin(CHART_MARGIN)
            .x_label_area_size(X_LABEL_AREA_SIZE)
            .y_label_area_size(Y_LABEL_AREA_SIZE)
            .build_cartesian_2d(-0.5f64..2.5f64, 0.0..y_max)?;

        chart
            .configure_mesh()
            .y_desc("Latency (ms)")
            .y_label_formatter(&|y| format!("{:.0}", y))
            .x_label_formatter(&|x| {
                let idx = (*x + 0.5) as usize;
                percentile_labels.get(idx).unwrap_or(&"").to_string()
            })
            .x_labels(3)
            .x_label_style((FONT_FAMILY, LABEL_FONT_SIZE))
            .y_label_style((FONT_FAMILY, LABEL_FONT_SIZE))
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        // Calculate bar positions
        let group_width = BAR_WIDTH_RATIO;
        let bar_width = group_width / num_gateways as f64;

        for (gateway_idx, (gateway_name, trend_values)) in gateway_data.iter().enumerate() {
            let color = color_map[gateway_name];

            // Calculate offset for this gateway's bars within each group
            let offset = -group_width / 2.0 + bar_width * (gateway_idx as f64 + 0.5);

            let values = vec![
                (0.0, trend_values.med),
                (1.0, trend_values.p95),
                (2.0, trend_values.p99),
            ];

            // Draw bars for this gateway
            let bars = values.iter().map(|(x, y)| {
                Rectangle::new(
                    [
                        (x + offset - bar_width / 2.0, 0.0),
                        (x + offset + bar_width / 2.0, *y),
                    ],
                    ShapeStyle::from(color).filled(),
                )
            });

            chart.draw_series(bars)?;

            // Draw value labels at 45 degrees
            for (x, y) in &values {
                let label_x = x + offset;
                let label_y = *y + (y_max * VALUE_LABEL_Y_OFFSET_RATIO); // Slightly above the bar

                // Parameterize decimal places
                let decimal_places = if *y < 100.0 { 1 } else { 0 };
                let label_text = format!("{:.prec$}", y, prec = decimal_places);

                chart.draw_series(std::iter::once(Text::new(
                    label_text,
                    (label_x, label_y),
                    (FONT_FAMILY, VALUE_FONT_SIZE)
                        .into_font()
                        .transform(FontTransform::Rotate270)
                        .color(&BLACK),
                )))?;
            }
        }

        // Draw legend manually in the legend area
        // Convert TrendValues to BenchmarkResult references for draw_legend
        // Exclude gateways with failures
        let legend_data: Vec<(&str, &BenchmarkResult)> = results
            .iter()
            .filter(|r| !r.has_failures() && r.k6_run.summary.metrics.http_req_duration.is_some())
            .map(|r| (r.gateway.as_str(), *r))
            .collect();
        draw_legend(&legend_area, &legend_data, &color_map)?;

        root.present()?;
    }

    Ok(buffer)
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
    use crate::charts::tests::*;

    #[test]
    fn test_generate_latency_chart_excludes_failures() {
        let results = vec![
            BenchmarkResult {
                scenario: "test-scenario".to_string(),
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
            // Gateway B has failures and should be excluded
            BenchmarkResult {
                scenario: "test-scenario".to_string(),
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
                                values: HttpReqFailedValues { fails: 10 }, // Has failures
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

        // Should contain Gateway A
        assert!(svg.contains("Gateway A"));
        // Should NOT contain Gateway B (has failures)
        assert!(!svg.contains("Gateway B"));
    }

    #[test]
    fn test_generate_latency_chart() {
        let results = vec![
            BenchmarkResult {
                scenario: "test-scenario".to_string(),
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
                scenario: "test-scenario".to_string(),
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
        assert!(svg.contains("Test Scenario - latencies"));
        assert!(svg.contains("Gateway A"));
        assert!(svg.contains("Gateway B"));
    }
}
