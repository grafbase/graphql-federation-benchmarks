# System Information

- Date: 2025-09-05
- CPU: AMD Ryzen 9 7950X3D 16-Core Processor
- Memory: 93.4 GiB
- CPU Boost: Disabled
- Git Commit: 58c000d814dfbd4237219c396fb72412b0fea1de
- Linux Version: 6.16.1
- Docker Version: 28.3.3

# many-plans

We use 7 subgraphs with very similar schemas and execute a fairly large and deep query retrieving all possible fields.
This forces the gateway query planner to consider many different possible plans as each individual field can be resolved by multiple
subgraphs and every object is an entity that allows for entity joins.

The goal being to measure how efficient query planning is, this scenario is only relevant for gateways that have caching disabled.
We only really care about the query planning performance and how many subgraph requests end up being executed. The subgraph requests
themselves are so small and simple that they shouldn't have any significant impact.

Query Planning performance is important during the re-deployment of gateways where many plans need to be re-computed.

K6 runs with a single VU.

## Latencies (ms)

![Latency Chart](charts/many-plans-latency.svg)

| Gateway                     |    Min |    Med |    P90 |    P95 |    P99 |    Max |
| :-------------------------- | -----: | -----: | -----: | -----: | -----: | -----: |
| Grafbase Gateway            |    1.7 |    2.1 |    2.3 |    2.4 |    2.6 |   28.1 |
| Apollo Router               |    8.2 |    9.2 |   10.0 |   10.3 |   12.1 | 3410.7 |
| Cosmo Router                |   15.6 |   18.1 |   19.6 |   20.0 |   21.2 |  392.6 |
| Grafbase Gateway (no cache) |   18.5 |   19.5 |   20.5 |   21.4 |   22.5 |   28.1 |
| Cosmo Router (no cache)     |  358.3 |  375.5 |  382.6 |  388.2 |  390.6 |  390.7 |
| Apollo Router (no cache)    | 3338.5 | 3344.1 | 3390.6 | 3396.4 | 3401.0 | 3402.2 |
| Apollo Router (with dedup)  | errors | errors | errors | errors | errors | errors |
| Hive Gateway                | errors | errors | errors | errors | errors | errors |
| Hive Gateway (no cache?)    | errors | errors | errors | errors | errors | errors |
| Hive Router                 |   >13s |   >13s |   >13s |   >13s |   >13s |   >13s |

## Resources

![Efficiency Chart](charts/many-plans-efficiency.svg)

| Gateway                     |       CPU | CPU max |       Memory |  MEM max | requests/core.s | requests/GB.s |
| :-------------------------- | --------: | ------: | -----------: | -------: | --------------: | ------------: |
| Grafbase Gateway            |  167% ±1% |    168% |    45 ±2 MiB |   48 MiB |           278.1 |        9925.4 |
| Grafbase Gateway (no cache) |  108% ±0% |    108% |    98 ±6 MiB |  106 MiB |            46.7 |         486.1 |
| Apollo Router               | 152% ±34% |    174% | 494 ±118 MiB |  800 MiB |            40.3 |          89.7 |
| Cosmo Router                |  504% ±5% |    512% |    71 ±2 MiB |   73 MiB |            10.3 |         742.1 |
| Cosmo Router (no cache)     | 160% ±11% |    179% |    74 ±6 MiB |   80 MiB |             1.5 |          34.2 |
| Apollo Router (no cache)    |   99% ±1% |    101% | 745 ±283 MiB | 1235 MiB |             0.3 |           0.2 |
| Apollo Router (with dedup)  | 145% ±29% |    164% | 481 ±125 MiB |  803 MiB |          errors |        errors |
| Hive Gateway                |  110% ±5% |    121% |  475 ±83 MiB |  536 MiB |          errors |        errors |
| Hive Gateway (no cache?)    |  109% ±5% |    121% |  474 ±84 MiB |  537 MiB |          errors |        errors |
| Hive Router                 |  100% ±0% |    100% |   138 ±0 MiB |  138 MiB |             0.0 |           0.0 |

## Requests

![Quality Chart](charts/many-plans-quality.svg)

| Gateway                     | Requests | Failures | Subgraph requests (total) |
| :-------------------------- | -------: | -------: | ------------------------: |
| Grafbase Gateway            |     4683 |        0 |             77.7 (363842) |
| Grafbase Gateway (no cache) |      506 |        0 |              77.8 (39351) |
| Cosmo Router (no cache)     |       27 |        0 |                189 (5091) |
| Cosmo Router                |      529 |        0 |              193 (101867) |
| Apollo Router               |      701 |        0 |              203 (142303) |
| Apollo Router (no cache)    |        3 |        0 |                 203 (609) |
| Apollo Router (with dedup)  |      754 |        3 |               108 (81265) |
| Hive Gateway                |      679 |      679 |               6.00 (4074) |
| Hive Gateway (no cache?)    |      680 |      680 |               6.00 (4080) |
| Hive Router                 |        0 |        0 |                     0 (0) |
