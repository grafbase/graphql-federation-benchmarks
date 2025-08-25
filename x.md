plan.md │ │
│ │ │ │
│ │ # GraphQL Federation Benchmarking System Plan │ │
│ │ │ │
│ │ ## Overview │ │
│ │ A Rust CLI tool that orchestrates GraphQL federation benchmarks, monitoring resource usage and collecting performance metrics for multiple gateway implementations. │ │
│ │ │ │
│ │ ## Architecture │ │
│ │ │ │
│ │ ### CLI Structure │ │
│ │ Single Rust binary with integrated modules: │ │
│ │ - **main**: CLI entry point using `argh` for argument parsing │ │
│ │ - **orchestrator**: Manages benchmark execution workflow │ │
│ │ - **metrics**: Collects CPU/memory stats from Docker containers │ │
│ │ - **docker**: Wrapper around `duct` for Docker operations │ │
│ │ - **results**: Formats and exports benchmark results │ │
│ │ │ │
│ │ ## Benchmark Structure │ │
│ │ │ │
│ │ `                                                                                                                                                                                       │ │
│ │ benchmarks/                                                                                                                                                                                │ │
│ │ └── [benchmark-name]/                                                                                                                                                                      │ │
│ │     ├── compose.yml             # Docker compose for subgraphs                                                                                                                             │ │
│ │     ├── k6.js                   # K6 load test script                                                                                                                                      │ │
│ │     ├── request/                                                                                                                                                                           │ │
│ │     │   ├── body.json          # Request payload                                                                                                                                           │ │
│ │     │   ├── expected.json      # Expected response                                                                                                                                         │ │
│ │     │   └── query.graphql      # GraphQL query                                                                                                                                             │ │
│ │     ├── subgraph/              # Rust subgraph implementation                                                                                                                              │ │
│ │     │   ├── Cargo.toml                                                                                                                                                                     │ │
│ │     │   └── src/                                                                                                                                                                           │ │
│ │     └── gateways/                                                                                                                                                                          │ │
│ │         ├── config.toml        # Gateway configurations                                                                                                                                    │ │
│ │         └── [gateway-name]/    # Gateway-specific files                                                                                                                                    │ │
│ │             ├── schema.graphql                                                                                                                                                             │ │
│ │             ├── [config-files]                                                                                                                                                             │ │
│ │             └── ...                                                                                                                                                                        │ │
│ │` │ │
│ │ │ │
│ │ ## Gateway Configuration Format │ │
│ │ │ │
│ │ `gateways/config.toml`: │ │
│ │ `toml                                                                                                                                                                                    │ │
│ │ [[gateways.grafbase]]                                                                                                                                                                      │ │
│ │ image = "ghcr.io/grafbase/gateway:latest"                                                                                                                                                  │ │
│ │ arguments = [                                                                                                                                                                              │ │
│ │     "--schema",                                                                                                                                                                            │ │
│ │     "/data/schema.graphql",                                                                                                                                                                │ │
│ │     "--config",                                                                                                                                                                            │ │
│ │     "/data/grafbase.toml",                                                                                                                                                                 │ │
│ │ ]                                                                                                                                                                                          │ │
│ │ # Optional environment variables                                                                                                                                                           │ │
│ │ environment = { LOG_LEVEL = "info" }                                                                                                                                                       │ │
│ │                                                                                                                                                                                            │ │
│ │ [[gateways.apollo_router]]                                                                                                                                                                 │ │
│ │ image = "ghcr.io/apollographql/router:latest"                                                                                                                                              │ │
│ │ arguments = [                                                                                                                                                                              │ │
│ │     "--supergraph",                                                                                                                                                                        │ │
│ │     "/data/supergraph.graphql",                                                                                                                                                            │ │
│ │     "--config",                                                                                                                                                                            │ │
│ │     "/data/router.yaml",                                                                                                                                                                   │ │
│ │ ]                                                                                                                                                                                          │ │
│ │ environment = { APOLLO_TELEMETRY_DISABLED = "true" }                                                                                                                                       │ │
│ │ ` │ │
│ │ │ │
│ │ ## Implementation Details │ │
│ │ │ │
│ │ ### 1. CLI Commands │ │
│ │ │ │
│ │ `bash                                                                                                                                                                                    │ │
│ │ # Run a specific benchmark with a specific gateway                                                                                                                                         │ │
│ │ cargo run -- run --benchmark many-possible-query-plans --gateway grafbase                                                                                                                  │ │
│ │                                                                                                                                                                                            │ │
│ │ # Run a benchmark with all configured gateways                                                                                                                                             │ │
│ │ cargo run -- run --benchmark many-possible-query-plans --all-gateways                                                                                                                      │ │
│ │                                                                                                                                                                                            │ │
│ │ # Run all benchmarks with all gateways                                                                                                                                                     │ │
│ │ cargo run -- run --all                                                                                                                                                                     │ │
│ │                                                                                                                                                                                            │ │
│ │ # List available benchmarks and gateways                                                                                                                                                   │ │
│ │ cargo run -- list                                                                                                                                                                          │ │
│ │ ` │ │
│ │ │ │
│ │ ### 2. Execution Flow │ │
│ │ │ │
│ │ 1. **Setup Phase** │ │
│ │ - Parse CLI arguments using `argh` │ │
│ │ - Load benchmark configuration │ │
│ │ - Validate benchmark directory structure │ │
│ │ - Parse `gateways/config.toml` │ │
│ │ │ │
│ │ 2. **Subgraph Startup** │ │
│ │ - Use `duct` to run `docker compose up -d --wait` in benchmark directory │ │
│ │ - The `--wait` flag ensures subgraphs are healthy before continuing │ │
│ │ - Compose file includes health checks │ │
│ │ │ │
│ │ 3. **Gateway Startup** │ │
│ │ - For each gateway in config: │ │
│ │ - Start container with: │ │
│ │ - Network mode: host │ │
│ │ - Mount `gateways/[gateway-name]/` → `/data` │ │
│ │ - Environment variables from config │ │
│ │ - Command arguments from config │ │
│ │ - Health check: Send GraphQL query `query { __typename }` │ │
│ │ - Verify response is `{"data":{"__typename":"Query"}}` │ │
│ │ │ │
│ │ 4. **Metrics Collection** │ │
│ │ - Start background task using `bollard` Docker API │ │
│ │ - Poll container stats every second │ │
│ │ - Track for each container: │ │
│ │ - CPU percentage │ │
│ │ - Memory usage (RSS) │ │
│ │ - Memory limit │ │
│ │ - Network I/O │ │
│ │ - Store time-series data │ │
│ │ │ │
│ │ 5. **Load Test Execution** │ │
│ │ - Run k6 test using `duct`: │ │
│ │ `rust                                                                                                                                                                               │ │
│ │      cmd!("k6", "run", "--out", "json=results.json", "k6.js")                                                                                                                              │ │
│ │      ` │ │
│ │ - Continue metrics collection throughout test │ │
│ │ - Parse k6 JSON output │ │
│ │ │ │
│ │ 6. **Cleanup** │ │
│ │ - Stop gateway container │ │
│ │ - Run `docker compose down` for subgraphs │ │
│ │ - Ensure all containers are removed │ │
│ │ │ │
│ │ 7. **Results Processing** │ │
│ │ - Parse k6 results (requests/sec, latencies, errors) │ │
│ │ - Calculate resource usage statistics │ │
│ │ - Generate JSON output │ │
│ │ │ │
│ │ ### 3. Metrics Collection Module │ │
│ │ │ │
│ │ `rust                                                                                                                                                                                    │ │
│ │ use bollard::Docker;                                                                                                                                                                       │ │
│ │ use jiff::{Timestamp, Span};                                                                                                                                                               │ │
│ │                                                                                                                                                                                            │ │
│ │ pub struct MetricsCollector {                                                                                                                                                              │ │
│ │     docker: Docker,                                                                                                                                                                        │ │
│ │     interval: Span,                                                                                                                                                                        │ │
│ │     samples: Vec<ResourceSample>,                                                                                                                                                          │ │
│ │ }                                                                                                                                                                                          │ │
│ │                                                                                                                                                                                            │ │
│ │ pub struct ResourceSample {                                                                                                                                                                │ │
│ │     timestamp: Timestamp,                                                                                                                                                                  │ │
│ │     container_name: String,                                                                                                                                                                │ │
│ │     cpu_percent: f64,                                                                                                                                                                      │ │
│ │     memory_bytes: u64,                                                                                                                                                                     │ │
│ │     memory_limit: u64,                                                                                                                                                                     │ │
│ │ }                                                                                                                                                                                          │ │
│ │                                                                                                                                                                                            │ │
│ │ impl MetricsCollector {                                                                                                                                                                    │ │
│ │     pub async fn start_collection(&mut self, container_ids: Vec<String>);                                                                                                                  │ │
│ │     pub async fn stop_collection(&mut self) -> MetricsReport;                                                                                                                              │ │
│ │                                                                                                                                                                                            │ │
│ │     async fn collect_stats(&mut self, container_id: &str) -> Result<()> {                                                                                                                  │ │
│ │         // Use bollard to get container stats                                                                                                                                              │ │
│ │         let stats = self.docker.stats(container_id, Some(false)).await?;                                                                                                                   │ │
│ │         // Process and store stats                                                                                                                                                         │ │
│ │     }                                                                                                                                                                                      │ │
│ │ }                                                                                                                                                                                          │ │
│ │ ` │ │
│ │ │ │
│ │ ### 4. Docker Integration │ │
│ │ │ │
│ │ `rust                                                                                                                                                                                    │ │
│ │ use duct::cmd;                                                                                                                                                                             │ │
│ │                                                                                                                                                                                            │ │
│ │ pub struct DockerManager;                                                                                                                                                                  │ │
│ │                                                                                                                                                                                            │ │
│ │ impl DockerManager {                                                                                                                                                                       │ │
│ │     pub fn compose_up(&self, path: &Path) -> Result<()> {                                                                                                                                  │ │
│ │         cmd!("docker", "compose", "up", "-d", "--wait")                                                                                                                                    │ │
│ │             .dir(path)                                                                                                                                                                     │ │
│ │             .run()?;                                                                                                                                                                       │ │
│ │         Ok(())                                                                                                                                                                             │ │
│ │     }                                                                                                                                                                                      │ │
│ │                                                                                                                                                                                            │ │
│ │     pub fn run_gateway(                                                                                                                                                                    │ │
│ │         &self,                                                                                                                                                                             │ │
│ │         gateway_name: &str,                                                                                                                                                                │ │
│ │         config: &GatewayConfig,                                                                                                                                                            │ │
│ │         benchmark_path: &Path                                                                                                                                                              │ │
│ │     ) -> Result<String> {                                                                                                                                                                  │ │
│ │         let gateway_path = benchmark_path.join("gateways").join(gateway_name);                                                                                                             │ │
│ │                                                                                                                                                                                            │ │
│ │         let mut args = vec![                                                                                                                                                               │ │
│ │             "run", "-d",                                                                                                                                                                   │ │
│ │             "--network", "host",                                                                                                                                                           │ │
│ │             "-v", &format!("{}:/data", gateway_path.display()),                                                                                                                            │ │
│ │         ];                                                                                                                                                                                 │ │
│ │                                                                                                                                                                                            │ │
│ │         // Add environment variables                                                                                                                                                       │ │
│ │         for (key, value) in &config.environment {                                                                                                                                          │ │
│ │             args.push("-e");                                                                                                                                                               │ │
│ │             args.push(&format!("{}={}", key, value));                                                                                                                                      │ │
│ │         }                                                                                                                                                                                  │ │
│ │                                                                                                                                                                                            │ │
│ │         args.push(&config.image);                                                                                                                                                          │ │
│ │         args.extend(&config.arguments);                                                                                                                                                    │ │
│ │                                                                                                                                                                                            │ │
│ │         let container_id = cmd("docker", &args).read()?;                                                                                                                                   │ │
│ │         Ok(container_id.trim().to_string())                                                                                                                                                │ │
│ │     }                                                                                                                                                                                      │ │
│ │                                                                                                                                                                                            │ │
│ │     pub fn compose_down(&self, path: &Path) -> Result<()> {                                                                                                                                │ │
│ │         cmd!("docker", "compose", "down", "-v")                                                                                                                                            │ │
│ │             .dir(path)                                                                                                                                                                     │ │
│ │             .run()?;                                                                                                                                                                       │ │
│ │         Ok(())                                                                                                                                                                             │ │
│ │     }                                                                                                                                                                                      │ │
│ │ }                                                                                                                                                                                          │ │
│ │ ` │ │
│ │ │ │
│ │ ### 5. Gateway Health Check │ │
│ │ │ │
│ │ `rust                                                                                                                                                                                    │ │
│ │ async fn check_gateway_health(port: u16) -> Result<bool> {                                                                                                                                 │ │
│ │     let client = reqwest::Client::new();                                                                                                                                                   │ │
│ │     let query = r#"{"query":"query { __typename }"}"#;                                                                                                                                     │ │
│ │                                                                                                                                                                                            │ │
│ │     let response = client                                                                                                                                                                  │ │
│ │         .post(&format!("http://localhost:{}/graphql", port))                                                                                                                               │ │
│ │         .header("Content-Type", "application/json")                                                                                                                                        │ │
│ │         .body(query)                                                                                                                                                                       │ │
│ │         .send()                                                                                                                                                                            │ │
│ │         .await?;                                                                                                                                                                           │ │
│ │                                                                                                                                                                                            │ │
│ │     let json: serde_json::Value = response.json().await?;                                                                                                                                  │ │
│ │                                                                                                                                                                                            │ │
│ │     Ok(json["data"]["__typename"] == "Query")                                                                                                                                              │ │
│ │ }                                                                                                                                                                                          │ │
│ │ ` │ │
│ │ │ │
│ │ ### 6. Output Format │ │
│ │ │ │
│ │ JSON output structure: │ │
│ │ `json                                                                                                                                                                                    │ │
│ │ {                                                                                                                                                                                          │ │
│ │   "benchmark": "many-possible-query-plans",                                                                                                                                                │ │
│ │   "gateway": "grafbase",                                                                                                                                                                   │ │
│ │   "timestamp": "2024-01-15T10:30:00Z",                                                                                                                                                     │ │
│ │   "duration_seconds": 20,                                                                                                                                                                  │ │
│ │   "k6_metrics": {                                                                                                                                                                          │ │
│ │     "iterations": 5000,                                                                                                                                                                    │ │
│ │     "iterations_per_sec": 250.0,                                                                                                                                                           │ │
│ │     "data_received": "150MB",                                                                                                                                                              │ │
│ │     "data_sent": "50MB",                                                                                                                                                                   │ │
│ │     "http_req_duration": {                                                                                                                                                                 │ │
│ │       "avg": 15.2,                                                                                                                                                                         │ │
│ │       "min": 5.1,                                                                                                                                                                          │ │
│ │       "med": 14.8,                                                                                                                                                                         │ │
│ │       "max": 125.3,                                                                                                                                                                        │ │
│ │       "p90": 25.4,                                                                                                                                                                         │ │
│ │       "p95": 45.8,                                                                                                                                                                         │ │
│ │       "p99": 89.3                                                                                                                                                                          │ │
│ │     },                                                                                                                                                                                     │ │
│ │     "http_req_failed": 0.0                                                                                                                                                                 │ │
│ │   },                                                                                                                                                                                       │ │
│ │   "resource_metrics": {                                                                                                                                                                    │ │
│ │     "gateway": {                                                                                                                                                                           │ │
│ │       "cpu": {                                                                                                                                                                             │ │
│ │         "avg": 45.2,                                                                                                                                                                       │ │
│ │         "min": 12.1,                                                                                                                                                                       │ │
│ │         "max": 78.9,                                                                                                                                                                       │ │
│ │         "p95": 72.3                                                                                                                                                                        │ │
│ │       },                                                                                                                                                                                   │ │
│ │       "memory_mb": {                                                                                                                                                                       │ │
│ │         "avg": 256.4,                                                                                                                                                                      │ │
│ │         "min": 245.1,                                                                                                                                                                      │ │
│ │         "max": 312.8,                                                                                                                                                                      │ │
│ │         "p95": 298.5                                                                                                                                                                       │ │
│ │       }                                                                                                                                                                                    │ │
│ │     },                                                                                                                                                                                     │ │
│ │     "subgraphs": {                                                                                                                                                                         │ │
│ │       "subgraph": {                                                                                                                                                                        │ │
│ │         "cpu": {                                                                                                                                                                           │ │
│ │           "avg": 12.3,                                                                                                                                                                     │ │
│ │           "max": 24.5                                                                                                                                                                      │ │
│ │         },                                                                                                                                                                                 │ │
│ │         "memory_mb": {                                                                                                                                                                     │ │
│ │           "avg": 128.5,                                                                                                                                                                    │ │
│ │           "max": 145.2                                                                                                                                                                     │ │
│ │         }                                                                                                                                                                                  │ │
│ │       }                                                                                                                                                                                    │ │
│ │     }                                                                                                                                                                                      │ │
│ │   }                                                                                                                                                                                        │ │
│ │ }                                                                                                                                                                                          │ │
│ │ ` │ │
│ │ │ │
│ │ ## Dependencies │ │
│ │ │ │
│ │ `toml                                                                                                                                                                                    │ │
│ │ [dependencies]                                                                                                                                                                             │ │
│ │ argh = "0.1"                # CLI argument parsing                                                                                                                                         │ │
│ │ tokio = { version = "1", features = ["full"] }                                                                                                                                             │ │
│ │ bollard = "0.16"           # Docker API client                                                                                                                                             │ │
│ │ duct = "0.13"              # Process execution                                                                                                                                             │ │
│ │ serde = { version = "1", features = ["derive"] }                                                                                                                                           │ │
│ │ serde_json = "1"                                                                                                                                                                           │ │
│ │ toml = "0.8"                                                                                                                                                                               │ │
│ │ jiff = "0.1"               # Time handling                                                                                                                                                 │ │
│ │ anyhow = "1"                                                                                                                                                                               │ │
│ │ reqwest = { version = "0.11", features = ["json"] }                                                                                                                                        │ │
│ │ tracing = "0.1"                                                                                                                                                                            │ │
│ │ tracing-subscriber = "0.3"                                                                                                                                                                 │ │
│ │ ` │ │
│ │ │ │
│ │ ## CLI Interface with argh │ │
│ │ │ │
│ │ `rust                                                                                                                                                                                    │ │
│ │ use argh::FromArgs;                                                                                                                                                                        │ │
│ │                                                                                                                                                                                            │ │
│ │ #[derive(FromArgs)]                                                                                                                                                                        │ │
│ │ /// GraphQL Federation Benchmark Runner                                                                                                                                                    │ │
│ │ struct Cli {                                                                                                                                                                               │ │
│ │     #[argh(subcommand)]                                                                                                                                                                    │ │
│ │     command: Command,                                                                                                                                                                      │ │
│ │ }                                                                                                                                                                                          │ │
│ │                                                                                                                                                                                            │ │
│ │ #[derive(FromArgs)]                                                                                                                                                                        │ │
│ │ #[argh(subcommand)]                                                                                                                                                                        │ │
│ │ enum Command {                                                                                                                                                                             │ │
│ │     Run(RunCommand),                                                                                                                                                                       │ │
│ │     List(ListCommand),                                                                                                                                                                     │ │
│ │ }                                                                                                                                                                                          │ │
│ │                                                                                                                                                                                            │ │
│ │ #[derive(FromArgs)]                                                                                                                                                                        │ │
│ │ #[argh(subcommand, name = "run")]                                                                                                                                                          │ │
│ │ /// Run benchmarks                                                                                                                                                                         │ │
│ │ struct RunCommand {                                                                                                                                                                        │ │
│ │     /// benchmark name to run                                                                                                                                                              │ │
│ │     #[argh(option, short = 'b')]                                                                                                                                                           │ │
│ │     benchmark: Option<String>,                                                                                                                                                             │ │
│ │                                                                                                                                                                                            │ │
│ │     /// gateway name to test                                                                                                                                                               │ │
│ │     #[argh(option, short = 'g')]                                                                                                                                                           │ │
│ │     gateway: Option<String>,                                                                                                                                                               │ │
│ │                                                                                                                                                                                            │ │
│ │     /// run all benchmarks                                                                                                                                                                 │ │
│ │     #[argh(switch)]                                                                                                                                                                        │ │
│ │     all: bool,                                                                                                                                                                             │ │
│ │                                                                                                                                                                                            │ │
│ │     /// run all gateways for the benchmark                                                                                                                                                 │ │
│ │     #[argh(switch)]                                                                                                                                                                        │ │
│ │     all_gateways: bool,                                                                                                                                                                    │ │
│ │                                                                                                                                                                                            │ │
│ │     /// output file path (default: stdout)                                                                                                                                                 │ │
│ │     #[argh(option, short = 'o')]                                                                                                                                                           │ │
│ │     output: Option<PathBuf>,                                                                                                                                                               │ │
│ │ }                                                                                                                                                                                          │ │
│ │                                                                                                                                                                                            │ │
│ │ #[derive(FromArgs)]                                                                                                                                                                        │ │
│ │ #[argh(subcommand, name = "list")]                                                                                                                                                         │ │
│ │ /// List available benchmarks and gateways                                                                                                                                                 │ │
│ │ struct ListCommand {}                                                                                                                                                                      │ │
│ │ ` │ │
│ │ │ │
│ │ ## Development Phases │ │
│ │ │ │
│ │ ### Phase 1: Core Infrastructure │ │
│ │ - CLI skeleton with `argh` │ │
│ │ - Benchmark discovery and validation │ │
│ │ - Configuration parsing │ │
│ │ │ │
│ │ ### Phase 2: Docker Integration │ │
│ │ - Docker compose operations via `duct` │ │
│ │ - Gateway container management │ │
│ │ - Health check implementation │ │
│ │ │ │
│ │ ### Phase 3: Metrics Collection │ │
│ │ - Docker Stats API integration with `bollard` │ │
│ │ - Background metrics collection task │ │
│ │ - Time-series data storage │ │
│ │ │ │
│ │ ### Phase 4: K6 Integration │ │
│ │ - K6 execution and output parsing │ │
│ │ - Result aggregation │ │
│ │ - JSON output generation │ │
│ │ │ │
│ │ ### Phase 5: Polish │ │
│ │ - Error handling and cleanup │ │
│ │ - Progress indicators │ │
│ │ - Logging with `tracing` │ │
│ │ - Parallel gateway testing │ │
│ │ │ │
│ │ ## Error Handling │ │
│ │ │ │
│ │ - Graceful cleanup on interruption (Ctrl+C) │ │
│ │ - Automatic container cleanup in Drop implementations │ │
│ │ - Detailed error messages with context │ │
│ │ - Retry logic for gateway health checks │ │
│ │ │ │
│ │ ## Project Structure │ │
│ │ │ │
│ │ `                                                                                                                                                                                       │ │
│ │ graphql-federation-benchmarks/                                                                                                                                                             │ │
│ │ ├── Cargo.toml                                                                                                                                                                             │ │
│ │ ├── src/                                                                                                                                                                                   │ │
│ │ │   ├── main.rs           # CLI entry point                                                                                                                                                │ │
│ │ │   ├── cli.rs            # argh command definitions                                                                                                                                       │ │
│ │ │   ├── orchestrator.rs   # Benchmark execution logic                                                                                                                                      │ │
│ │ │   ├── docker.rs         # Docker operations                                                                                                                                              │ │
│ │ │   ├── metrics.rs        # Resource monitoring                                                                                                                                            │ │
│ │ │   ├── k6.rs            # K6 integration                                                                                                                                                  │ │
│ │ │   └── results.rs       # Output formatting                                                                                                                                               │ │
│ │ └── benchmarks/          # Benchmark definitions                                                                                                                                           │ │
│ │` │ │
│ │ │ │
│ │ ## Testing Strategy │ │
│ │ │ │
│ │ 1. Unit tests for each module │ │
│ │ 2. Integration tests with Docker │ │
│ │ 3. End-to-end test with sample benchmark │ │
│ │ 4. Mock tests for Docker API calls
