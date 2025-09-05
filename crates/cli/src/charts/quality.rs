use super::*;
use crate::benchmark::BenchmarkResult;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

pub fn generate_quality_chart(
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

        // Split the drawing area: title + charts on the left, legend on the right
        let (main_area, legend_area) = root.split_horizontally(CHART_WIDTH - legend_width);

        // Split main area into title and chart areas
        let (title_area, chart_area) = main_area.split_vertically(40);

        // Add title centered in the title area
        let title_text = format!("{} - quality", scenario_name);
        let title_style = TextStyle::from((FONT_FAMILY, TITLE_FONT_SIZE).into_font())
            .pos(Pos::new(HPos::Center, VPos::Center));
        title_area.draw(&Text::new(
            title_text,
            (title_area.dim_in_pixel().0 as i32 / 2, 20),
            title_style,
        ))?;

        // Create vector of gateway names and data (will be sorted in draw_quality_panel)
        // Exclude gateways with failures
        let gateway_data: Vec<(&str, &BenchmarkResult)> = results
            .iter()
            .filter(|r| r.is_valid())
            .map(|r| (r.gateway.as_str(), *r))
            .collect();

        // Create color mapping based on alphabetically sorted gateway names
        let color_map = create_color_map(results);

        // Draw single panel for subgraph requests (sorted lowest to highest since lower is better)
        draw_quality_panel(
            &chart_area,
            &gateway_data,
            &color_map,
            "Average Subgraph Requests",
            calculate_avg_subgraph_requests,
        )?;

        // Draw legend manually in the legend area
        draw_legend(&legend_area, &gateway_data, &color_map)?;

        root.present()?;
    }

    Ok(buffer)
}

// Similar to draw_efficiency_panel but sorts from lowest to highest (for metrics where lower is better)
fn draw_quality_panel<F>(
    area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
    gateway_data: &[(&str, &BenchmarkResult)],
    color_map: &HashMap<&str, RGBColor>,
    caption: &str,
    value_fn: F,
) -> anyhow::Result<()>
where
    F: Fn(&BenchmarkResult) -> f64,
{
    use plotters::style::IntoFont;

    // Create sorted data for this specific metric (lowest to highest - lower is better)
    let mut sorted_data: Vec<_> = gateway_data
        .iter()
        .map(|(name, result)| {
            let value = value_fn(result);
            (*name, result, value)
        })
        .collect();
    sorted_data.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    let max_value = sorted_data
        .iter()
        .map(|(_, _, v)| *v)
        .fold(0.0f64, |acc, val| acc.max(val));
    let y_max = (max_value * 1.1).ceil();

    let num_gateways = gateway_data.len();
    let x_range = -0.5f64..(num_gateways as f64 - 0.5);

    // Add caption at the top of the panel
    let (caption_area, chart_area) = area.split_vertically(30);
    caption_area.draw(&Text::new(
        caption,
        (caption_area.dim_in_pixel().0 as i32 / 2, 15),
        TextStyle::from((FONT_FAMILY, CAPTION_FONT_SIZE).into_font())
            .pos(Pos::new(HPos::Center, VPos::Center)),
    ))?;

    let mut chart = ChartBuilder::on(&chart_area)
        .margin(PANEL_MARGIN)
        .x_label_area_size(0) // No x-label area
        .y_label_area_size(Y_LABEL_AREA_SIZE_SMALL)
        .build_cartesian_2d(x_range, 0.0..y_max)?;

    chart
        .configure_mesh()
        .y_label_formatter(&|y| {
            if *y >= KILO_THRESHOLD {
                format!("{:.0}k", y / KILO_THRESHOLD)
            } else {
                format!("{:.0}", y)
            }
        })
        .x_labels(0) // No x-axis labels
        .y_label_style((FONT_FAMILY, LABEL_FONT_SIZE))
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    // Draw bars using sorted data
    let bar_width = BAR_WIDTH_RATIO;

    for (idx, (gateway_name, _result, value)) in sorted_data.iter().enumerate() {
        let color = color_map[gateway_name];

        chart.draw_series(std::iter::once(Rectangle::new(
            [
                (idx as f64 - bar_width / 2.0, 0.0),
                (idx as f64 + bar_width / 2.0, *value),
            ],
            ShapeStyle::from(color).filled(),
        )))?;

        // Draw value label
        let decimal_places = if *value < 10.0 { 1 } else { 0 };
        let label_text = if *value >= KILO_THRESHOLD {
            format!("{:.prec$}k", value / KILO_THRESHOLD, prec = decimal_places)
        } else {
            format!("{:.prec$}", value, prec = decimal_places)
        };

        chart.draw_series(std::iter::once(Text::new(
            label_text,
            (idx as f64, value + y_max * VALUE_LABEL_Y_OFFSET_RATIO),
            (FONT_FAMILY, VALUE_FONT_SIZE)
                .into_font()
                .transform(FontTransform::Rotate270)
                .color(&BLACK),
        )))?;
    }

    Ok(())
}

pub fn generate_quality_chart_to_file(
    scenario_name: &str,
    results: &[&BenchmarkResult],
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let svg_content = generate_quality_chart(scenario_name, results)?;
    std::fs::write(output_path, svg_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::tests::*;

    #[test]
    fn test_generate_quality_chart() {
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
                        subgraph_stats: SubgraphStats { count: 500 },
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
                        subgraph_stats: SubgraphStats { count: 600 },
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
                    cpu_usage_max: 1.2,
                    cpu_usage_std: 0.15,
                    memory_mib_avg: 300.0,
                    memory_mib_max: 1024.0,
                    memory_mib_std: 60.0,
                    throttled_time: Duration::from_secs(0),
                    count: 100,
                },
            },
        ];

        let refs: Vec<&BenchmarkResult> = results.iter().collect();
        let svg = generate_quality_chart("Test Scenario", &refs).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Scenario - quality"));
        assert!(svg.contains("Average Subgraph Requests"));
        assert!(svg.contains("Gateway A"));
        assert!(svg.contains("Gateway B"));
    }
}
