# System Information

Date: 2025-09-05
CPU: AMD Ryzen 9 7950X3D 16-Core Processor
Memory: 93.4 GiB
CPU Boost: Disabled
Git Commit: 6e2f6ce9149792b5d10de3263431aef7ea947695
Linux Version: 6.16.1
Docker Version: 28.3.3
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

| Gateway                     |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :-------------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Grafbase Gateway            |     1.7 |     2.1 |     2.4 |     2.5 |     2.8 |    30.7 |
| Apollo Router               |     8.1 |     9.2 |    10.0 |    10.3 |    11.8 |  3393.5 |
| Cosmo Router                |    15.4 |    17.9 |    19.3 |    19.7 |    20.4 |   389.2 |
| Grafbase Gateway (no cache) |    18.6 |    19.6 |    20.7 |    21.3 |    22.5 |    26.2 |
| Cosmo Router (no cache)     |   361.8 |   374.8 |   383.7 |   384.6 |   385.0 |   385.1 |
| Apollo Router (no cache)    |  3331.7 |  3355.7 |  3387.8 |  3391.8 |  3395.0 |  3395.8 |
| Apollo Router (with dedup)  |     err |     err |     err |     err |     err |     err |
| Hive Gateway                |     err |     err |     err |     err |     err |     err |
| Hive Gateway (no cache?)    |     err |     err |     err |     err |     err |     err |
| Hive Router                 |    >13s |    >13s |    >13s |    >13s |    >13s |    >13s |

## Resources

![Efficiency Chart](charts/many-plans-efficiency.svg)

| Gateway                     |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :-------------------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Grafbase Gateway            |     167% ±1% |     169% |      45 ±1 MiB |    48 MiB |            271.0 |         9729.9 |
| Grafbase Gateway (no cache) |     108% ±0% |     108% |      97 ±4 MiB |   104 MiB |             46.6 |          494.6 |
| Apollo Router               |    152% ±34% |     175% |   508 ±113 MiB |   803 MiB |             40.4 |           90.2 |
| Cosmo Router                |     502% ±6% |     514% |      71 ±2 MiB |    75 MiB |             10.4 |          728.0 |
| Cosmo Router (no cache)     |     159% ±8% |     173% |      70 ±6 MiB |    77 MiB |              1.5 |           35.6 |
| Apollo Router (no cache)    |      99% ±1% |     101% |   739 ±276 MiB |  1212 MiB |              0.3 |            0.3 |
| Apollo Router (with dedup)  |    145% ±29% |     164% |   468 ±131 MiB |   803 MiB |              err |            err |
| Hive Gateway                |     109% ±5% |     121% |    475 ±84 MiB |   537 MiB |              err |            err |
| Hive Gateway (no cache?)    |     110% ±5% |     121% |    475 ±83 MiB |   536 MiB |              err |            err |
| Hive Router                 |     100% ±0% |     100% |     137 ±1 MiB |   139 MiB |              0.0 |            0.0 |

## Requests

![Quality Chart](charts/many-plans-quality.svg)

| Gateway                     | Requests | Failures | Subgraph requests (total) |
| :-------------------------- | -------: | -------: | ------------------------: |
| Grafbase Gateway (no cache) |      503 |        0 |              77.7 (39071) |
| Grafbase Gateway            |     4574 |        0 |             77.7 (355464) |
| Cosmo Router (no cache)     |       27 |        0 |                190 (5118) |
| Cosmo Router                |      533 |        0 |              192 (102372) |
| Apollo Router               |      708 |        0 |              203 (143724) |
| Apollo Router (no cache)    |        3 |        0 |                 203 (609) |
| Apollo Router (with dedup)  |      747 |        2 |               108 (80497) |
| Hive Gateway                |      661 |      661 |               6.00 (3966) |
| Hive Gateway (no cache?)    |      666 |      666 |               6.00 (3996) |
| Hive Router                 |        0 |        0 |                     0 (0) |

