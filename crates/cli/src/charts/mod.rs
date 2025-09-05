mod efficiency;
mod latency;
mod quality;

use efficiency::generate_efficiency_chart_to_file;
use latency::generate_latency_chart_to_file;
use quality::generate_quality_chart_to_file;

use crate::benchmark::BenchmarkResult;
use crate::config::Config;
use plotters::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Write all charts for the benchmark results to the specified directory
pub fn write_charts(
    results: &[BenchmarkResult],
    _config: &Config,
    dir: &Path,
) -> anyhow::Result<()> {
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(dir)?;
    
    // Group results by benchmark name
    let mut grouped_results: BTreeMap<String, Vec<&BenchmarkResult>> = BTreeMap::new();
    for result in results {
        grouped_results
            .entry(result.benchmark.clone())
            .or_default()
            .push(result);
    }
    
    // Generate charts for each benchmark
    for (benchmark_name, benchmark_results) in grouped_results {
        // Generate latency chart
        let latency_filename = format!("{}-latency.svg", benchmark_name.replace(' ', "-"));
        let latency_path = dir.join(&latency_filename);
        generate_latency_chart_to_file(
            &benchmark_name,
            &benchmark_results,
            &latency_path,
        )?;
        
        // Generate efficiency chart
        let efficiency_filename = format!("{}-efficiency.svg", benchmark_name.replace(' ', "-"));
        let efficiency_path = dir.join(&efficiency_filename);
        generate_efficiency_chart_to_file(
            &benchmark_name,
            &benchmark_results,
            &efficiency_path,
        )?;
        
        // Generate quality chart
        let quality_filename = format!("{}-quality.svg", benchmark_name.replace(' ', "-"));
        let quality_path = dir.join(&quality_filename);
        generate_quality_chart_to_file(
            &benchmark_name,
            &benchmark_results,
            &quality_path,
        )?;
    }
    
    Ok(())
}

// Chart dimensions
const CHART_WIDTH: u32 = 900; // Increased to accommodate legend on the side
const CHART_HEIGHT: u32 = 600;
const LEGEND_WIDTH: u32 = 195; // Space for legend on the right (increased by 30%)

// Colors
const TRANSPARENT_BACKGROUND: RGBAColor = RGBAColor(255, 255, 255, 0.0);
const GATEWAY_COLORS: &[RGBColor] = &[
    RGBColor(7, 168, 101),   // #07A865 - green
    RGBColor(30, 144, 255),  // #1E90FF - dodger blue
    RGBColor(231, 150, 243), // #E796F3 - light purple/pink
    RGBColor(223, 104, 45),  // #DF682D - burnt orange
    RGBColor(189, 229, 108), // #BDE56C - light green
    RGBColor(158, 177, 255), // #9EB1FF - light blue
];

// Font settings
const FONT_FAMILY: &str = "sans-serif";
const TITLE_FONT_SIZE: i32 = 30;
const CAPTION_FONT_SIZE: i32 = 20;
const LEGEND_FONT_SIZE: i32 = 18;
const LABEL_FONT_SIZE: i32 = 16;
const VALUE_FONT_SIZE: i32 = 16;

// Layout settings
const CHART_MARGIN: i32 = 20;
const PANEL_MARGIN: i32 = 10;
const X_LABEL_AREA_SIZE: i32 = 50;
const Y_LABEL_AREA_SIZE: i32 = 70;
const Y_LABEL_AREA_SIZE_SMALL: i32 = 50; // For panels without x-labels
const LEGEND_Y_START: i32 = 60;
const LEGEND_ITEM_HEIGHT: i32 = 25;
const LEGEND_BOX_X: i32 = 5;
const LEGEND_BOX_SIZE: i32 = 15;
const LEGEND_TEXT_X: i32 = 25;
const LEGEND_TEXT_Y_OFFSET: i32 = 3;

// Bar chart settings
const BAR_WIDTH_RATIO: f64 = 0.8;
const VALUE_LABEL_Y_OFFSET_RATIO: f64 = 0.02; // Offset as ratio of y_max

// Data formatting
const KILO_THRESHOLD: f64 = 1000.0;

/// Create color mapping based on alphabetically sorted gateway names
fn create_color_map<'a>(results: &[&'a BenchmarkResult]) -> HashMap<&'a str, RGBColor> {
    let mut gateway_names: Vec<&str> = results.iter().map(|r| r.gateway.as_str()).collect();
    gateway_names.sort();
    gateway_names.dedup();
    
    gateway_names
        .iter()
        .enumerate()
        .map(|(idx, name)| (*name, GATEWAY_COLORS[idx % GATEWAY_COLORS.len()]))
        .collect()
}

/// Draw legend for a chart
fn draw_legend(
    legend_area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
    gateway_data: &[(&str, &BenchmarkResult)],
    color_map: &HashMap<&str, RGBColor>,
) -> anyhow::Result<()> {
    let legend_y_start = LEGEND_Y_START;
    let legend_item_height = LEGEND_ITEM_HEIGHT;

    for (idx, (gateway_name, _)) in gateway_data.iter().enumerate() {
        let color = color_map[gateway_name];
        let y_pos = legend_y_start + (idx as i32 * legend_item_height);

        // Draw color box (moved closer to chart)
        legend_area.draw(&Rectangle::new(
            [(LEGEND_BOX_X, y_pos), (LEGEND_BOX_X + LEGEND_BOX_SIZE, y_pos + LEGEND_BOX_SIZE)],
            color.filled(),
        ))?;

        // Draw gateway name
        legend_area.draw(&Text::new(
            gateway_name.to_string(),
            (LEGEND_TEXT_X, y_pos + LEGEND_TEXT_Y_OFFSET),
            (FONT_FAMILY, LEGEND_FONT_SIZE).into_font(),
        ))?;
    }
    
    Ok(())
}

/// Calculate request rate from benchmark result
fn calculate_request_rate(result: &BenchmarkResult) -> f64 {
    let duration_ms = result.k6_run.summary.state.test_run_duration_ms;
    let requests_count = result.k6_run.summary.metrics.http_req_duration
        .as_ref()
        .map(|m| m.values.count)
        .unwrap_or(0) as f64;
    requests_count / (duration_ms / 1000.0)
}

/// Calculate average subgraph requests per gateway request
fn calculate_avg_subgraph_requests(result: &BenchmarkResult) -> f64 {
    let requests_count = result.k6_run.summary.metrics.http_req_duration
        .as_ref()
        .map(|m| m.values.count)
        .unwrap_or(1) as f64;
    result.k6_run.summary.subgraph_stats.count as f64 / requests_count
}

#[cfg(test)]
pub mod tests {
    pub use crate::k6::{
        CheckMetric, HttpReqFailedValues, K6Run, K6Summary, K6SummaryMetrics, K6SummaryState,
        SubgraphStats, TrendMetric, TrendValues,
    };
    pub use crate::resources::ResourceStats;
    pub use std::time::Duration;
}