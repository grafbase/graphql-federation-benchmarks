# GraphQL Federation Gateway Benchmarks

This repo contains a list of complex benchmark cases to measure the performance of GraphQL Federation gateways.

## Latest report

[report (2025-09-05)](./REPORT.md)

## Methodology

Every service, gateway and subgraphs, are running in docker containers with `--network host` to avoid any overhead.
Usually, a single service exposing all the subgrpahs composing a supergraph. Subgraphs are optimized for speed and serve responses mostly from cache.

Every request has a unique `authorization` header value which is propagated (except for Hive Router 0.0.8 which seems to ignore it as of 2025-09-05) to the subgraph.
This ensures that gateways don't abuse the repetitive nature of the benchmarks. It's only repetitive because:

- it's hard to generate good non-repetitive workloads.
- it provides a lot of data points for a scenario.

So unless specified the goal is not to test how a gateway behaves against recurrent queries.

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

Be warned that those commands will stop and delete _all_ docker containers without any mercy.
You can adjust this behavior in the `docker-clean.sh` script. But, sometimes the clean up is needed...

```bash
# Run all benchmarks with all gateways
./cli.sh run

# Run specific benchmark with specific gateway
./cli.sh bench --scenario many-plans --gateway grafbase
```
