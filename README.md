# GraphQL Federation Gateway Benchmarks

This repo contains a list of complex benchmark cases to measure the performance of GraphQL Federation gateways.

## Benchmarks

### big-response

A single subgraph that returns a 4MiB response.
The goal is to measure how much overhead the gateway adds parsing and validating and incoming response.

K6 runs for 30s with a single VU, executing requests sequentially.

### many-plans

The supergraph consists of 7 subgraphs, served by a single server, are exposing very similar schemas. The query is constructed to have many different possible plans. The goal is to measure how efficient the query planer is at finding a good solution and how much time it takes.
A good solution is one that minimizes the number of subgraph request.

K6 runs for 30s with a single VU, executing requests sequentially.

For Apollo Router, we had to fork the project to disable the plan caching as the whole point of this benchmark is to measure the planning performance. The code used is there: https://github.com/Finistere/router/tree/qp-disable-cache

## Protocol

The gateway and the subgraphs are run through docker with `--network host` to avoid any overhead. The load testing and latency measure is done by K6.
The subgraphs keep track of the number of incoming GraphQL requests, excluding health checks. K6 retrieves any statistics computed by the subgraph at the end and propagates it in its summary. During the tests, we track the resource usage (CPU & MEM) of the gateway through `docker stats`. We only keep the measurements when K6 was running.

A report is provided at the end.

## Latest report

```
# System Information

Date: 2025-08-26
CPU: AMD Ryzen 9 7950X3D 16-Core Processor
Memory: 93.4 GiB
CPU Boost: Disabled

# Benchmarks

## big-response

### Requests

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Apollo Router    |      359 |        0 |                   1 (359) |
| Grafbase Gateway |      790 |        0 |                   1 (790) |

### Latencies (ms)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router    |    73.9 |    82.6 |    88.4 |    90.6 |    96.1 |   118.9 |
| Grafbase Gateway |    34.7 |    37.1 |    40.6 |    41.8 |    44.1 |    56.0 |

### Resources

| Gateway          |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :--------------- | -------: | -------: | --------: | --------: |
| Apollo Router    |      71% |      73% |   484 MiB |   539 MiB |
| Grafbase Gateway |      38% |      39% |   142 MiB |   164 MiB |

## many-plans

### Requests

| Gateway                  | Requests | Failures | Subgraph requests (total) |
| :----------------------- | -------: | -------: | ------------------------: |
| Apollo Router (no cache) |        9 |        0 |                203 (1827) |
| Grafbase Gateway         |     1432 |        0 |               83 (118856) |

### Latencies (ms)

| Gateway                  |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :----------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router (no cache) |  3313.2 |  3328.8 |  3371.2 |  3381.7 |  3390.2 |  3392.3 |
| Grafbase Gateway         |    18.7 |    20.9 |    22.1 |    22.4 |    23.3 |    55.5 |

### Resources

| Gateway                  |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :----------------------- | -------: | -------: | --------: | --------: |
| Apollo Router (no cache) |     100% |     101% |  1335 MiB |  2434 MiB |
| Grafbase Gateway         |     107% |     108% |   200 MiB |   241 MiB |
```

## Running the benchmarks

### Requirements

- Docker and Docker Compose
- Rust toolchain (rustup)
- K6 load testing tool

### CPU boost

CPU boost can skew the results as frequencis will be higher when few CPU cores are used. You can disable it on Linux with:

```sh
echo "0" | sudo tee /sys/devices/system/cpu/cpufreq/boost
```

### Commands

```bash
# Run all benchmarks with all gateways
cargo run --release -- run

# Run specific benchmark with specific gateway
cargo run --release -- run --benchmark many-possible-query-plans --gateway grafbase

# List available benchmarks and gateways
cargo run --release -- list
```
