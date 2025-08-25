use serde::Serialize;

use crate::k6::K6Run;
use crate::resources::ResourceStats;

#[derive(Debug, Serialize)]
pub struct BenchmarkResult<'a> {
    pub benchmark: String,
    pub gateway: &'a str,
    pub k6_run: K6Run,
    pub resource_stats: ResourceStats,
}
