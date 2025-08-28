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

Date: 2025-08-28
CPU: AMD Ryzen 9 7950X3D 16-Core Processor
Memory: 93.4 GiB
CPU Boost: Disabled

# Benchmarks

## big-response

### Requests

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Apollo Router    |      365 |        0 |                   1 (365) |
| Cosmo Router     |      465 |        0 |                   1 (465) |
| Grafbase Gateway |      786 |        0 |                   1 (786) |

### Latencies (ms)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router    |    71.2 |    81.3 |    86.9 |    90.0 |    94.9 |   124.0 |
| Cosmo Router     |    53.2 |    63.7 |    70.0 |    72.4 |    76.8 |    90.6 |
| Grafbase Gateway |    34.7 |    37.2 |    40.7 |    41.7 |    44.3 |    56.8 |

### Resources

| Gateway          |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :--------------- | -------: | -------: | --------: | --------: |
| Apollo Router    |      71% |      73% |   491 MiB |   591 MiB |
| Cosmo Router     |     148% |     159% |    83 MiB |   107 MiB |
| Grafbase Gateway |      38% |      40% |   158 MiB |   172 MiB |

## many-plans

### Requests

| Gateway                     | Requests | Failures | Subgraph requests (total) |
| :-------------------------- | -------: | -------: | ------------------------: |
| Apollo Router (no cache)    |        9 |        0 |                203 (1827) |
| Cosmo Router (no cache)     |       73 |        0 |               155 (11344) |
| Grafbase Gateway (no cache) |     1441 |        0 |               83 (119603) |

### Latencies (ms)

| Gateway                     |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :-------------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router (no cache)    |  3332.8 |  3345.3 |  3387.0 |  3408.9 |  3426.5 |  3430.9 |
| Cosmo Router (no cache)     |   401.3 |   411.3 |   416.1 |   418.3 |   423.2 |   424.9 |
| Grafbase Gateway (no cache) |    18.7 |    20.8 |    21.9 |    22.3 |    23.1 |    56.4 |

### Resources

| Gateway                     |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :-------------------------- | -------: | -------: | --------: | --------: |
| Apollo Router (no cache)    |     100% |     101% |  1392 MiB |  2444 MiB |
| Cosmo Router (no cache)     |     154% |     173% |    75 MiB |    84 MiB |
| Grafbase Gateway (no cache) |     107% |     108% |   174 MiB |   226 MiB |
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

### Troubleshoot

If you encounter errors, you might need to clean your running containers:

```sh
 docker stop $(docker ps -a -q) -t 2 && docker rm $(docker ps -a -q)
```
