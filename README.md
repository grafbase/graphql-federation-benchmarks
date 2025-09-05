# GraphQL Federation Gateway Benchmarks

This repo contains a list of complex benchmark cases to measure the performance of GraphQL Federation gateways.

## Latest report

[report (2025-09-05)](./REPORT.md)

## Protocol

Every service, gateway and subgraphs, are running in docker containers with `--network host` to avoid any overhead.
Usually, a single service exposing all the subgrpahs composing a supergraph. Subgraphs are optimized for speed and serve responses mostly from cache.

Every request has a unique `authorization` header value which is propagated, except for Hive Router as of 2025-09-05, to the subgraph. This ensures that gateways don't abuse the repetitive nature of the benchmark. Unless explicitly specified scenarios are not testing how good a gateway can de-duplicate requests.

The load testing itself is done with K6. Multiple scenarios have been created to benchmark different situations.

We measure the following:

| Metric                | Source                                              |
| --------------------- | --------------------------------------------------- |
| Response latencies    | K6                                                  |
| Response count & rate | K6                                                  |
| Subgraph requests     | Subgraph service\* (retrieved by K6 at the end)     |
| CPU                   | `cpu_stats.cpu_usage.total_usage` from docker stats |
| Memory                | `memory_stats.usage` from docker stats              |

\* health checks are excluded.

A report is provided at the end with all the numerical results. Charts are also generated, but we only use the data from successful benchmark runs. Gateways that have errors or don't return a response are grayed out. Whatever we measured is not comparable.

## Running the benchmarks

### Requirements

- Docker and Docker Compose
- Rust toolchain (rustup)
- K6 load testing tool

### CPU boost

CPU boost can skew the results as frequencies will be higher when few CPU cores are used. You can disable it on Linux with:

```sh
echo "0" | sudo tee /sys/devices/system/cpu/cpufreq/boost
```

### Commands

Be warned that those commands will stop and delete _all_ docker containers without any mercy. If you don't want that, adjust the `docker-clean.sh` script. But for a reason that escapes me for now, sometimes it's needed.

```bash
# Run all benchmarks with all gateways
./cli.sh run

# Run specific benchmark with specific gateway
./cli.sh bench --scenario many-plans --gateway grafbase
```

### Troubleshoot

If you encounter errors, you might need to clean your running containers:

```sh
 docker stop $(docker ps -a -q) -t 2 && docker rm $(docker ps -a -q)
```
