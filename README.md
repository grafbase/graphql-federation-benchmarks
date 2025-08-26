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
| Apollo Router    |      354 |        0 |                   1 (354) |
| Grafbase Gateway |      779 |        0 |                   1 (779) |

### Latencies (ms)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router    |    76.3 |    83.7 |    90.1 |    92.8 |    95.8 |   118.3 |
| Grafbase Gateway |    31.8 |    37.7 |    41.0 |    42.3 |    46.1 |    58.5 |

### Resources

| Gateway          |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :--------------- | -------: | -------: | --------: | --------: |
| Apollo Router    |      71% |      73% |   451 MiB |   529 MiB |
| Grafbase Gateway |      37% |      39% |   150 MiB |   166 MiB |

## many-plans

### Requests

| Gateway                | Requests | Failures | Subgraph requests (total) |
| :--------------------- | -------: | -------: | ------------------------: |
| Apollo Router (custom) |        9 |        0 |                203 (1827) |
| Grafbase Gateway       |     1417 |        0 |               83 (117611) |

### Latencies (ms)

| Gateway                |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router (custom) |  3327.8 |  3359.4 |  3394.4 |  3409.8 |  3422.1 |  3425.1 |
| Grafbase Gateway       |    18.6 |    21.1 |    22.4 |    22.8 |    23.6 |    59.3 |

### Resources

| Gateway                |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :--------------------- | -------: | -------: | --------: | --------: |
| Apollo Router (custom) |     100% |     101% |  1353 MiB |  2443 MiB |
| Grafbase Gateway       |     107% |     108% |   175 MiB |   208 MiB |
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
