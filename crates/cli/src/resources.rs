use anyhow::Result;
use bollard::{Docker, query_parameters::StatsOptionsBuilder, secret::ContainerStatsResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::docker::ContainerId;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    pub cpu_usage_avg: f64,
    pub cpu_usage_max: f64,
    pub cpu_usage_std: f64,
    pub memory_mib_avg: f64,
    pub memory_mib_max: f64,
    pub memory_mib_std: f64,
    pub throttled_time: Duration,
    pub count: usize,
}

pub struct DockerStatsCollector {
    is_collecting: Arc<AtomicBool>,
    handle: Option<JoinHandle<Vec<StatSample>>>,
}

impl DockerStatsCollector {
    pub async fn start(docker: Docker, container_id: &ContainerId) -> anyhow::Result<Self> {
        let is_collecting = Arc::new(AtomicBool::new(true));
        let is_collecting_clone = is_collecting.clone();

        let mut samples = Vec::new();
        let mut stats_stream = docker.stats(
            container_id,
            Some(StatsOptionsBuilder::new().stream(true).build()),
        );

        let handle = tokio::spawn(async move {
            while is_collecting_clone.load(Ordering::SeqCst) {
                tokio::select! {
                    stat_result = futures_util::StreamExt::next(&mut stats_stream) => {
                        if let Some(Ok(stats)) = stat_result
                            && let Ok(sample) = stats.try_into() {
                                samples.push(sample);
                            }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        continue;
                    }
                }
            }

            samples
        });

        Ok(Self {
            is_collecting,
            handle: Some(handle),
        })
    }

    pub async fn stop_and_filter(
        mut self,
        start: time::OffsetDateTime,
        end: time::OffsetDateTime,
    ) -> Result<ResourceStats> {
        // Signal collection to stop
        self.is_collecting.store(false, Ordering::SeqCst);

        // Wait briefly for final samples
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Get samples from collection task
        let handle = self
            .handle
            .take()
            .ok_or_else(|| anyhow::anyhow!("Collection handle not available"))?;

        let samples = handle
            .await
            .map_err(|e| anyhow::anyhow!("Failed to join metrics collection task: {}", e))?;

        assert!(samples.is_sorted_by_key(|s| s.read) && samples.is_sorted_by_key(|s| s.preread));
        let start_ix = samples.partition_point(|s| s.preread < start);

        let mut stats = ResourceStats::default();
        let mut cpu_values = Vec::new();
        let mut memory_values = Vec::new();

        for sample in &samples[start_ix..] {
            if sample.read > end {
                break;
            }

            stats.count += 1;

            let cpu_usage = (sample.cpu_total_usage - sample.precpu_total_usage)
                .div_duration_f64((sample.read - sample.preread).try_into().unwrap());
            cpu_values.push(cpu_usage);
            stats.cpu_usage_avg += (cpu_usage - stats.cpu_usage_avg) / (stats.count as f64);
            stats.cpu_usage_max = stats.cpu_usage_max.max(cpu_usage);

            let memory_mib = sample.memory_bytes as f64 / ((1 << 20) as f64);
            memory_values.push(memory_mib);
            if stats.memory_mib_max.total_cmp(&memory_mib).is_lt() {
                stats.memory_mib_max = memory_mib;
            }
            stats.memory_mib_avg += (memory_mib - stats.memory_mib_avg) / (stats.count as f64);

            stats.throttled_time += sample.throttled_time.unwrap_or_default();
        }

        // Calculate standard deviations using statrs
        use statrs::statistics::Statistics;
        if !cpu_values.is_empty() {
            stats.cpu_usage_std = cpu_values.std_dev();
        }
        if !memory_values.is_empty() {
            stats.memory_mib_std = memory_values.std_dev();
        }

        Ok(stats)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatSample {
    preread: time::OffsetDateTime,
    read: time::OffsetDateTime,
    precpu_total_usage: Duration,
    cpu_total_usage: Duration,
    memory_bytes: u64,
    throttled_time: Option<Duration>,
}

impl TryFrom<ContainerStatsResponse> for StatSample {
    // Example on Linux:
    // {
    //  "name": "/sharp_hoover",266 complete and 0 interrupted iterations
    //  "id": "df6732cfcd45464337faa72ccd8e991cb100a80b128c3ce6f537a01effccca3c",
    //  "read": "2025-08-25T19:55:59.963636756Z",
    //  "preread": "2025-08-25T19:55:58.961947717Z",
    //  "pids_stats": {
    //    "current": 35,
    //    "limit": 114615
    //  },
    //  "blkio_stats": {
    //    "io_service_bytes_recursive": [
    //      {
    //        "major": 259,
    //        "minor": 0,
    //        "op": "read",
    //        "value": 6471680
    //      },
    //      {
    //        "major": 259,
    //        "minor": 0,
    //        "op": "write",
    //        "value": 0
    //      },
    //      {
    //        "major": 254,
    //        "minor": 0,
    //        "op": "read",
    //        "value": 6471680
    //      },
    //      {
    //        "major": 254,
    //        "minor": 0,
    //        "op": "write",
    //        "value": 0
    //      }
    //    ]
    //  },
    //  "num_procs": 0,
    //  "storage_stats": {},
    //  "cpu_stats": {
    //    "cpu_usage": {
    //      "total_usage": 3103291000,
    //      "usage_in_kernelmode": 511518000,
    //      "usage_in_usermode": 2591773000
    //    },
    //    "system_cpu_usage": 1294386280000000,
    //    "online_cpus": 32,
    //    "throttling_data": {
    //      "periods": 0,
    //      "throttled_periods": 0,
    //      "throttled_time": 0
    //    }
    //  },
    //  "precpu_stats": {
    //    "cpu_usage": {
    //      "total_usage": 2047379000,
    //      "usage_in_kernelmode": 349597000,
    //      "usage_in_usermode": 1697782000
    //    },
    //    "system_cpu_usage": 1294354630000000,
    //    "online_cpus": 32,
    //    "throttling_data": {
    //      "periods": 0,
    //      "throttled_periods": 0,
    //      "throttled_time": 0
    //    }
    //  },
    //  "memory_stats": {
    //    "usage": 191705088,
    //    "stats": {
    //      "thp_collapse_alloc": 0,
    //      "workingset_nodereclaim": 0,
    //      "slab": 1071296,
    //      "pgdeactivate": 0,
    //      "pgactivate": 0,
    //      "unevictable": 0,
    //      "pgrefill": 0,
    //      "active_file": 5173248,
    //      "workingset_activate": 0,
    //      "pgmajfault": 42,
    //      "active_anon": 177098752,
    //      "file_writeback": 0,
    //      "anon_thp": 75497472,
    //      "shmem": 0,
    //      "inactive_file": 1298432,
    //      "file_mapped": 2002944,
    //      "file": 6471680,
    //      "sock": 0,
    //      "workingset_refault": 0,
    //      "pgfault": 75302,
    //      "pglazyfreed": 0,
    //      "kernel_stack": 573440,
    //      "pgsteal": 0,
    //      "slab_reclaimable": 544760,
    //      "file_dirty": 0,
    //      "slab_unreclaimable": 526536,
    //      "inactive_anon": 0,
    //      "pgscan": 0,
    //      "anon": 136564736,
    //      "pglazyfree": 0,
    //      "thp_fault_alloc": 465
    //    },
    //    "limit": 100307812352
    //  }
    //}
    type Error = anyhow::Error;
    fn try_from(resp: ContainerStatsResponse) -> anyhow::Result<Self> {
        let cpu_stats = resp
            .cpu_stats
            .ok_or_else(|| anyhow::anyhow!("CPU stats not available"))?;
        Ok(StatSample {
            preread: resp
                .preread
                .ok_or_else(|| anyhow::anyhow!("Preread timestamp not available"))?,
            read: resp
                .read
                .ok_or_else(|| anyhow::anyhow!("Read timestamp not available"))?,
            cpu_total_usage: cpu_stats
                .cpu_usage
                .and_then(|usage| usage.total_usage)
                .map(|total_usage| {
                    // Total CPU time consumed in nanoseconds (Linux) or 100's of nanoseconds (Windows).
                    if cfg!(target_os = "linux") {
                        // On Linux, Docker provides CPU usage in nanoseconds
                        Duration::from_nanos(total_usage)
                    } else {
                        // On other platforms, we may need a different calculation
                        Duration::from_nanos(total_usage * 100)
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("CPU usage not available"))?,
            precpu_total_usage: resp
                .precpu_stats
                .and_then(|st| st.cpu_usage)
                .and_then(|usage| usage.total_usage)
                .map(|total_usage| {
                    if cfg!(target_os = "linux") {
                        Duration::from_nanos(total_usage)
                    } else {
                        Duration::from_nanos(total_usage * 100)
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("Pre-CPU usage not available"))?,
            memory_bytes: resp
                .memory_stats
                .and_then(|st| st.usage)
                .ok_or_else(|| anyhow::anyhow!("Memory usage not available"))?,
            throttled_time: cpu_stats
                .throttling_data
                .and_then(|td| td.throttled_time.map(Duration::from_nanos)),
        })
    }
}
