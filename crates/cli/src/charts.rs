use crate::benchmark::BenchmarkResult;
use plotters::prelude::*;

const CHART_WIDTH: u32 = 900; // Increased to accommodate legend on the side
const CHART_HEIGHT: u32 = 600;
const LEGEND_WIDTH: u32 = 150; // Space for legend on the right

const GATEWAY_COLORS: &[RGBColor] = &[
    RGBColor(7, 168, 101),   // #07A865 - green
    RGBColor(30, 144, 255),  // #1E90FF - dodger blue
    RGBColor(231, 150, 243), // #E796F3 - light purple/pink
    RGBColor(223, 104, 45),  // #DF682D - burnt orange
    RGBColor(189, 229, 108), // #BDE56C - light green
    RGBColor(158, 177, 255), // #9EB1FF - light blue
];

pub fn generate_latency_chart(
    scenario_name: &str,
    results: &[&BenchmarkResult],
) -> anyhow::Result<String> {
    use plotters::style::IntoFont;

    let mut buffer = String::new();
    {
        let root =
            SVGBackend::with_string(&mut buffer, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();

        // Transparent background
        root.fill(&RGBColor(255, 255, 255).mix(0.0))?;

        // Split the drawing area: chart on the left, legend on the right
        let (chart_area, legend_area) = root.split_horizontally(CHART_WIDTH - LEGEND_WIDTH);

        // Create sorted vector of (gateway_name, TrendValues), sorted by median (lowest first)
        let mut gateway_data: Vec<(&str, &crate::k6::TrendValues)> = results
            .iter()
            .filter_map(|r| {
                r.k6_run.summary.metrics.http_req_duration.as_ref()
                    .map(|metric| (r.gateway.as_str(), &metric.values))
            })
            .collect();
        
        gateway_data.sort_by(|a, b| a.1.med.partial_cmp(&b.1.med).unwrap());
        
        // Create a color mapping based on alphabetically sorted gateway names for consistency
        let mut gateway_names: Vec<&str> = gateway_data.iter().map(|(name, _)| *name).collect();
        gateway_names.sort();
        let color_map: std::collections::HashMap<&str, RGBColor> = gateway_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (*name, GATEWAY_COLORS[idx % GATEWAY_COLORS.len()]))
            .collect();
        
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
                &format!("{} - Latency Distribution", scenario_name),
                ("sans-serif", 30).into_font(),
            )
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(60)
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
            .disable_x_mesh()
            .disable_y_mesh()
            .draw()?;

        // Calculate bar positions
        let group_width = 0.8;
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
                let label_y = *y + (y_max * 0.02); // Slightly above the bar

                // Parameterize decimal places
                let decimal_places = if *y < 100.0 { 1 } else { 0 };
                let label_text = format!("{:.prec$}", y, prec = decimal_places);

                chart.draw_series(std::iter::once(Text::new(
                    label_text,
                    (label_x, label_y),
                    ("sans-serif", 12)
                        .into_font()
                        .transform(FontTransform::Rotate270)
                        .color(&BLACK),
                )))?;
            }
        }

        // Draw legend manually in the legend area
        let legend_y_start = 60;
        let legend_item_height = 25;

        for (idx, (gateway_name, _)) in gateway_data.iter().enumerate() {
            let color = color_map[gateway_name];
            let y_pos = legend_y_start + (idx as i32 * legend_item_height);

            // Draw color box
            legend_area.draw(&Rectangle::new(
                [(10, y_pos), (25, y_pos + 15)],
                color.filled(),
            ))?;

            // Draw gateway name
            legend_area.draw(&Text::new(
                gateway_name.to_string(),
                (30, y_pos + 3),
                ("sans-serif", 14).into_font(),
            ))?;
        }

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
