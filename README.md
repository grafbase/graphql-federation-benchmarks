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

The gateway and the subgraphs are docker containers with `--network host` to avoid any overhead. The load testing and latency measure is done by K6.
The subgraphs keep track of the number of incoming GraphQL requests, excluding health checks. K6 retrieves any statistics computed by the subgraph at the end and propagates it in its summary. During the tests, we track the resource usage (CPU & MEM) of the gateway through `docker stats`. We only keep the measurements when K6 was running.

A report is provided at the end.

## Latest report

```
# System Information

Date: 2025-08-28
CPU: AMD Ryzen 9 7950X3D 16-Core Processor
Memory: 93.4 GiB
CPU Boost: Disabled
Git Commit: d729680366dc2050ed77c9d47381989fa7771542
Linux Version: 6.16.1
Docker Version: 28.3.3

# Benchmarks

## big-response

### Requests

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Apollo Router    |      369 |        0 |                   1 (369) |
| Cosmo Router     |      465 |        0 |                   1 (465) |
| Grafbase Gateway |      790 |        0 |                   1 (790) |
| Hive Gateway     |      286 |        0 |                   1 (286) |

### Latencies (ms)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router    |    73.4 |    80.3 |    85.8 |    87.9 |    99.3 |   117.4 |
| Cosmo Router     |    52.4 |    63.5 |    70.7 |    72.7 |    77.4 |    89.5 |
| Grafbase Gateway |    34.8 |    37.1 |    40.4 |    41.4 |    43.6 |    56.3 |
| Hive Gateway     |    98.9 |   103.2 |   107.8 |   110.3 |   132.7 |   199.5 |

### Resources

| Gateway          |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :--------------- | -------: | -------: | --------: | --------: |
| Apollo Router    |      71% |      73% |   484 MiB |   563 MiB |
| Cosmo Router     |     148% |     163% |    84 MiB |   118 MiB |
| Grafbase Gateway |      38% |      40% |   149 MiB |   169 MiB |
| Hive Gateway     |      98% |     102% |   434 MiB |   521 MiB |

## many-plans

### Requests

| Gateway                     | Requests | Failures | Subgraph requests (total) |
| :-------------------------- | -------: | -------: | ------------------------: |
| Apollo Router (no cache)    |        9 |        0 |                203 (1827) |
| Cosmo Router (no cache)     |       74 |        0 |               156 (11615) |
| Grafbase Gateway (no cache) |     1430 |        0 |               83 (118690) |

### Latencies (ms)

| Gateway                     |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :-------------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router (no cache)    |  3353.0 |  3358.2 |  3382.5 |  3404.7 |  3422.5 |  3426.9 |
| Cosmo Router (no cache)     |   400.0 |   410.1 |   417.9 |   419.6 |   422.4 |   424.5 |
| Grafbase Gateway (no cache) |    18.7 |    21.0 |    22.1 |    22.4 |    23.1 |    57.6 |

### Resources

| Gateway                     |  CPU avg |  CPU max |   MEM avg |   MEM max |
| :-------------------------- | -------: | -------: | --------: | --------: |
| Apollo Router (no cache)    |     100% |     101% |  1320 MiB |  2253 MiB |
| Cosmo Router (no cache)     |     155% |     170% |    74 MiB |    80 MiB |
| Grafbase Gateway (no cache) |     107% |     108% |   174 MiB |   210 MiB |
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
