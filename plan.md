# GraphQL Federation Benchmarking System Plan

## Overview

A Rust CLI tool that orchestrates GraphQL federation benchmarks, monitoring gateway resource usage during k6 load tests and collecting performance metrics.

## Architecture

### CLI Structure

Single Rust binary with integrated modules:

- **main**: CLI entry point using `argh` for argument parsing
- **orchestrator**: Manages benchmark execution workflow
- **metrics**: Collects CPU/memory stats from gateway container
- **docker**: Wrapper around `duct` for Docker operations
- **results**: Formats and exports benchmark results

## Benchmark Structure

```
benchmarks/
└── [benchmark-name]/
    ├── compose.yml             # Docker compose for subgraphs
    ├── k6.js                   # K6 load test script
    ├── request/
    │   ├── body.json          # Request payload
    │   ├── expected.json      # Expected response
    │   └── query.graphql      # GraphQL query
    ├── subgraph/              # Rust subgraph implementation
    │   ├── Cargo.toml
    │   └── src/
    └── gateways/
        ├── config.toml        # Gateway configurations
        └── [gateway-name]/    # Gateway-specific files
            ├── schema.graphql
            ├── [config-files]
            └── ...
```

## Gateway Configuration Format

`gateways/config.toml`:

```toml
[[gateways.grafbase]]
image = "ghcr.io/grafbase/gateway:latest"
arguments = [
    "--schema",
    "/data/schema.graphql",
    "--config",
    "/data/grafbase.toml",
]
# Optional environment variables
environment = { LOG_LEVEL = "info" }

[[gateways.apollo_router]]
image = "ghcr.io/apollographql/router:latest"
arguments = [
    "--supergraph",
    "/data/supergraph.graphql",
    "--config",
    "/data/router.yaml",
]
environment = { APOLLO_TELEMETRY_DISABLED = "true" }
```

## Implementation Details

### 1. CLI Commands

```bash
# Run specific benchmark with specific gateway
cargo run -- run --benchmark many-possible-query-plans --gateway grafbase

# Run all benchmarks with all gateways (no filters)
cargo run -- run

# Run all benchmarks with specific gateway
cargo run -- run --gateway grafbase

# Run specific benchmark with all gateways
cargo run -- run --benchmark many-possible-query-plans

# List available benchmarks and gateways
cargo run -- list
```

### 2. Execution Flow

1. **Setup Phase**

   - Parse CLI arguments using `argh`
   - Discover benchmarks based on filter
   - Parse `gateways/config.toml` for each benchmark
   - Filter gateways based on CLI argument

2. **For Each Benchmark/Gateway Combination:**

   a. **Subgraph Startup**

   - Use `duct` to run `docker compose up -d --wait`
   - The `--wait` flag ensures subgraphs are healthy

   b. **Gateway Startup**

   - Start gateway container with:
     - Network mode: host
     - Mount `gateways/[gateway-name]/` → `/data`
     - Environment variables from config
     - Command arguments from config
   - Health check: Send `query { __typename }`
   - Verify response contains `{"data":{"__typename":"Query"}}`

   c. **K6 Test with Metrics Collection**

   - Start metrics collector for gateway container
   - **Mark k6 start time**
   - Run k6 test using `duct`
   - **Mark k6 end time**
   - Stop metrics collector
   - Parse k6 JSON output

   d. **Cleanup**

   - Stop gateway container
   - Run `docker compose down` for subgraphs

   e. **Results Processing**

   - Filter metrics to k6 execution window
   - Calculate statistics for the test period
   - Generate JSON output

### 3. Metrics Collection Module

```rust
use bollard::Docker;
use jiff::{Timestamp, Span};

pub struct MetricsCollector {
    docker: Docker,
    container_id: String,
    interval: Span,
    samples: Vec<ResourceSample>,
    is_collecting: Arc<AtomicBool>,
}

pub struct ResourceSample {
    timestamp: Timestamp,
    cpu_percent: f64,
    memory_bytes: u64,
    memory_limit: u64,
}

pub struct MetricsReport {
    samples: Vec<ResourceSample>,
    k6_start: Timestamp,
    k6_end: Timestamp,
}

impl MetricsCollector {
    pub fn new(docker: Docker, container_id: String) -> Self {
        Self {
            docker,
            container_id,
            interval: Span::seconds(1),
            samples: Vec::new(),
            is_collecting: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start collecting metrics in background
    pub fn start_collection(&mut self) -> JoinHandle<()> {
        self.is_collecting.store(true, Ordering::SeqCst);
        // Spawn background task that polls Docker stats every second
    }

    /// Stop collection and return all samples
    pub fn stop_collection(&mut self) -> Vec<ResourceSample> {
        self.is_collecting.store(false, Ordering::SeqCst);
        // Return collected samples
        std::mem::take(&mut self.samples)
    }

    /// Filter samples to k6 execution window and calculate stats
    pub fn calculate_stats(
        samples: Vec<ResourceSample>,
        k6_start: Timestamp,
        k6_end: Timestamp,
    ) -> ResourceStats {
        let filtered: Vec<_> = samples
            .into_iter()
            .filter(|s| s.timestamp >= k6_start && s.timestamp <= k6_end)
            .collect();

        // Calculate min, max, avg, percentiles
        ResourceStats::from_samples(filtered)
    }
}
```

### 4. Docker Integration

```rust
use duct::cmd;

pub struct DockerManager;

impl DockerManager {
    pub fn compose_up(&self, path: &Path) -> Result<()> {
        cmd!("docker", "compose", "up", "-d", "--wait", "--build", "--force-recreate")
            .dir(path)
            .run()?;
        Ok(())
    }

    pub fn run_gateway(
        &self,
        gateway_name: &str,
        config: &GatewayConfig,
        benchmark_path: &Path
    ) -> Result<String> {
        let gateway_path = benchmark_path.join("gateways").join(gateway_name);

        let mut args = vec![
            "run", "-d",
            "--network", "host",
            "-v", &format!("{}:/data", gateway_path.display()),
        ];

        // Add environment variables
        for (key, value) in &config.environment {
            args.push("-e");
            args.push(&format!("{}={}", key, value));
        }

        args.push(&config.image);
        args.extend(&config.arguments);

        let container_id = cmd("docker", &args).read()?;
        Ok(container_id.trim().to_string())
    }

    pub fn stop_container(&self, container_id: &str) -> Result<()> {
        cmd!("docker", "stop", container_id).run()?;
        cmd!("docker", "rm", container_id).run()?;
        Ok(())
    }

    pub fn compose_down(&self, path: &Path) -> Result<()> {
        cmd!("docker", "compose", "down", "-v")
            .dir(path)
            .run()?;
        Ok(())
    }
}
```

### 5. K6 Execution with Timing

```rust
use jiff::Timestamp;

pub struct K6Runner;

impl K6Runner {
    pub async fn run_test(
        &self,
        benchmark_path: &Path,
        metrics_collector: &mut MetricsCollector,
    ) -> Result<K6Result> {
        // Start metrics collection
        let collection_handle = metrics_collector.start_collection();

        // Mark k6 start time
        let k6_start = Timestamp::now();

        // Run k6 test
        let output = cmd!(
            "k6", "run",
            "--out", "json=results.json",
            "k6.js"
        )
        .dir(benchmark_path)
        .run()?;

        // Mark k6 end time
        let k6_end = Timestamp::now();

        // Stop metrics collection
        let samples = metrics_collector.stop_collection();
        collection_handle.await?;

        // Parse k6 results
        let k6_metrics = self.parse_k6_output(benchmark_path.join("results.json"))?;

        // Calculate resource stats for k6 execution window
        let resource_stats = MetricsCollector::calculate_stats(
            samples,
            k6_start,
            k6_end,
        );

        Ok(K6Result {
            k6_metrics,
            resource_stats,
            k6_start,
            k6_end,
        })
    }
}
```

### 6. Output Format

```json
{
  "benchmark": "many-possible-query-plans",
  "gateway": "grafbase",
  "execution": {
    "start": "2024-01-15T10:30:00Z",
    "end": "2024-01-15T10:30:20Z",
    "duration_seconds": 20
  },
  "k6_metrics": {
    "iterations": 5000,
    "iterations_per_sec": 250.0,
    "data_received_bytes": 157286400,
    "data_sent_bytes": 52428800,
    "http_req_duration": {
      "avg": 15.2,
      "min": 5.1,
      "med": 14.8,
      "max": 125.3,
      "p90": 25.4,
      "p95": 45.8,
      "p99": 89.3
    },
    "http_req_failed_rate": 0.0
  },
  "gateway_resources": {
    "cpu_percent": {
      "avg": 45.2,
      "min": 12.1,
      "max": 78.9,
      "p50": 44.8,
      "p95": 72.3,
      "p99": 77.2
    },
    "memory_mb": {
      "avg": 256.4,
      "min": 245.1,
      "max": 312.8,
      "p50": 255.2,
      "p95": 298.5,
      "p99": 310.1
    },
    "sample_count": 20
  }
}
```

## Dependencies

```toml
[dependencies]
argh = "0.1"                # CLI argument parsing
tokio = { version = "1", features = ["full"] }
bollard = "0.16"           # Docker API client
duct = "0.13"              # Process execution
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
jiff = "0.1"               # Time handling
anyhow = "1"
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
statrs = "0.17"            # Statistical calculations
```

## CLI Interface with argh

```rust
use argh::FromArgs;

#[derive(FromArgs)]
/// GraphQL Federation Benchmark Runner
struct Cli {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Run(RunCommand),
    List(ListCommand),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run benchmarks
struct RunCommand {
    /// filter by benchmark name
    #[argh(option, short = 'b')]
    benchmark: Option<String>,

    /// filter by gateway name
    #[argh(option, short = 'g')]
    gateway: Option<String>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
/// List available benchmarks and gateways
struct ListCommand {}
```

## Orchestrator Logic

```rust
impl Orchestrator {
    pub async fn run(&self, benchmark_filter: Option<String>, gateway_filter: Option<String>) -> Result<()> {
        // Discover benchmarks
        let benchmarks = self.discover_benchmarks(benchmark_filter)?;

        for benchmark_path in benchmarks {
            // Parse gateway config for this benchmark
            let gateways = self.load_gateways(&benchmark_path, gateway_filter)?;

            for (gateway_name, gateway_config) in gateways {
                println!("Running benchmark '{}' with gateway '{}'",
                    benchmark_path.file_name().unwrap().to_str().unwrap(),
                    gateway_name
                );

                // Run single benchmark/gateway combination
                let result = self.run_single(
                    &benchmark_path,
                    &gateway_name,
                    &gateway_config
                ).await?;

                // Output result
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        }

        Ok(())
    }

    async fn run_single(
        &self,
        benchmark_path: &Path,
        gateway_name: &str,
        gateway_config: &GatewayConfig,
    ) -> Result<BenchmarkResult> {
        // 1. Start subgraphs
        self.docker.compose_up(benchmark_path)?;

        // 2. Start gateway
        let container_id = self.docker.run_gateway(
            gateway_name,
            gateway_config,
            benchmark_path
        )?;

        // 3. Wait for gateway health
        self.wait_for_gateway_health().await?;

        // 4. Create metrics collector for gateway
        let mut collector = MetricsCollector::new(
            self.docker.clone(),
            container_id.clone()
        );

        // 5. Run k6 with metrics collection
        let k6_result = K6Runner::new().run_test(
            benchmark_path,
            &mut collector
        ).await?;

        // 6. Cleanup
        self.docker.stop_container(&container_id)?;
        self.docker.compose_down(benchmark_path)?;

        // 7. Format result
        Ok(BenchmarkResult {
            benchmark: benchmark_path.file_name().unwrap().to_str().unwrap().to_string(),
            gateway: gateway_name.to_string(),
            execution: ExecutionInfo {
                start: k6_result.k6_start,
                end: k6_result.k6_end,
                duration_seconds: (k6_result.k6_end - k6_result.k6_start).as_seconds_f64(),
            },
            k6_metrics: k6_result.k6_metrics,
            gateway_resources: k6_result.resource_stats,
        })
    }
}
```

## Development Phases

### Phase 1: Core Infrastructure

- CLI skeleton with `argh`
- Benchmark discovery logic
- Gateway configuration parsing

### Phase 2: Docker Integration

- Docker compose operations via `duct`
- Gateway container lifecycle
- Health check implementation

### Phase 3: Metrics Collection

- Docker Stats API with `bollard`
- Timed collection around k6 execution
- Statistical calculations

### Phase 4: K6 Integration

- K6 execution with timing
- JSON output parsing
- Result formatting

### Phase 5: Polish

- Error handling and cleanup
- Progress indicators
- Concurrent benchmark execution
- Result aggregation and comparison

## Error Handling

- Graceful cleanup on interruption (Ctrl+C)
- Ensure containers are stopped/removed on failure
- Clear error messages with context
- Retry logic for gateway health checks (up to 30 seconds)

## Project Structure

```
graphql-federation-benchmarks/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── cli.rs            # argh command definitions
│   ├── orchestrator.rs   # Benchmark execution logic
│   ├── docker.rs         # Docker operations via duct
│   ├── metrics.rs        # Gateway resource monitoring
│   ├── k6.rs            # K6 execution and parsing
│   └── results.rs       # Output formatting
└── benchmarks/          # Benchmark definitions
```

## Key Design Decisions

1. **Single Container Monitoring**: Only track gateway container resources, not subgraphs
2. **Timed Metrics Window**: Collect metrics continuously but only analyze k6 execution period
3. **Filter-based CLI**: Simple benchmark/gateway filters instead of complex commands
4. **Automatic Discovery**: Find benchmarks and gateways based on directory structure
5. **Clean Separation**: Each module has a single responsibility
