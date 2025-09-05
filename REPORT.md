# System Information

Date: 2025-09-05
CPU: AMD Ryzen 9 7950X3D 16-Core Processor
Memory: 93.4 GiB
CPU Boost: Disabled
Git Commit: c2ef2b159882ea3d6b773dadbd4803df0683339c
Linux Version: 6.16.1
Docker Version: 28.3.3

# Benchmarks

## query

Fairly complex query requiring a dozen subgraph requests with some duplicate plans/requests. The goal here is to measure how well the gateways
behaves under a certain throughput.

K6 runs with a constant throughput of 100 requests/s


### Requests

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Apollo Router    |      501 |        0 |              16.00 (8016) |
| Cosmo Router     |      500 |        0 |               8.00 (4002) |
| Grafbase Gateway |      501 |        0 |              13.00 (6513) |

### Latencies (ms)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Apollo Router    |    45.1 |    47.9 |    49.2 |    49.7 |    50.5 |    53.0 |
| Cosmo Router     |    44.0 |    46.8 |    48.1 |    48.4 |    49.0 |    52.1 |
| Grafbase Gateway |    43.6 |    45.4 |    46.5 |    46.8 |    47.5 |    48.7 |

### Resources

| Gateway          |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :--------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Apollo Router    |      44% ±1% |      45% |      68 ±1 MiB |    69 MiB |            222.0 |         1471.7 |
| Cosmo Router     |      58% ±1% |      59% |      43 ±0 MiB |    44 MiB |            167.1 |         2334.2 |
| Grafbase Gateway |      12% ±0% |      13% |      31 ±1 MiB |    32 MiB |            774.8 |         3170.7 |

