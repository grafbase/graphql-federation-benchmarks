# GraphQL Federation Gateway Benchmarks

This repo contains a list of complex benchmark cases to measure the performance of GraphQL Federation gateways.

## Protocol

The gateway and the subgraphs are docker containers with `--network host` to avoid any overhead. The load testing and latency measure is done by K6.
The subgraphs keep track of the number of incoming GraphQL requests, excluding health checks. K6 retrieves any statistics computed by the subgraph at the end and propagates it in its summary. During the tests, we track the resource usage (CPU & MEM) of the gateway through `docker stats`. We only keep the measurements when K6 was running.

Every request has a unique `authorization` header value which is propagated to the subgraph. This ensures that gateways don't abuse the repetitive nature of the benchmark. Unless explicitly specified scenarios are not testing how good a gateway can de-duplicate requests.

A report is provided at the end.

## Latest report

[report](./REPORT.md)

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
