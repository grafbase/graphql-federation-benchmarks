# System Information

- Date: 2025-09-12
- CPU: AMD Ryzen 9 7950X3D 16-Core Processor
- Memory: 93.4 GiB
- CPU Boost: Disabled
- Git Commit: 386b93cb41494b0d83dd2dbd9b2b8380d73f51a6
- Linux Version: 6.16.1
- Docker Version: 28.3.3

# Gateways

The following gateways were tested (as configured in `config.toml`):

- apollo-router: ghcr.io/apollographql/router:v2.6.0
- apollo-router-dedup: ghcr.io/apollographql/router:v2.6.0
- apollo-router-no-cache: apollo-router-no-cache
- cosmo: ghcr.io/wundergraph/cosmo/router:0.252.1
- cosmo-no-cache: ghcr.io/wundergraph/cosmo/router:0.252.1
- grafbase: ghcr.io/grafbase/gateway:0.49.1
- grafbase-no-cache: ghcr.io/grafbase/gateway:0.49.1
- hive-gateway: ghcr.io/graphql-hive/gateway:2.1.2
- hive-router: ghcr.io/graphql-hive/router:0.0.9

# big-response

Tests gateway performance with large GraphQL response payloads (~8MiB) containing a mix of lists, objects strings, floats and ints.

K6 runs with a single VU, executing requests sequentially, to measure the best case latencies a gateway could provide.


## Latencies (ms)

![Latency Chart](charts/big-response-latency.svg)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Hive Router      |    21.4 |    26.3 |    30.6 |    31.5 |    33.8 |   128.8 |
| Grafbase Gateway |    26.0 |    29.4 |    32.3 |    33.4 |    35.4 |   133.0 |
| Cosmo Router     |    45.8 |    65.2 |    72.4 |    74.6 |    78.7 |   181.8 |
| Hive Gateway     |    95.4 |   100.3 |   109.8 |   111.3 |   116.0 |   324.1 |
| Apollo Router    |   108.8 |   122.4 |   127.9 |   130.5 |   134.5 |   241.7 |

## Resources

![Efficiency Chart](charts/big-response-efficiency.svg)

| Gateway          |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :--------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Hive Router      |      81% ±1% |      83% |     180 ±4 MiB |   189 MiB |             41.7 |          187.4 |
| Grafbase Gateway |      86% ±1% |      88% |      71 ±6 MiB |    82 MiB |             35.5 |          392.7 |
| Apollo Router    |      96% ±0% |      96% |    487 ±50 MiB |   602 MiB |              8.3 |           13.6 |
| Hive Gateway     |     120% ±2% |     124% |    930 ±20 MiB |   968 MiB |              7.7 |           10.1 |
| Cosmo Router     |    248% ±18% |     302% |    107 ±18 MiB |   147 MiB |              4.9 |          103.6 |

## Requests

![Quality Chart](charts/big-response-quality.svg)

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Apollo Router    |      481 |        0 |                1.00 (481) |
| Cosmo Router     |      894 |        0 |                1.00 (894) |
| Grafbase Gateway |     1883 |        0 |               1.00 (1883) |
| Hive Gateway     |      573 |        0 |                1.00 (573) |
| Hive Router      |     2075 |        0 |               1.00 (2075) |

# deduplication

Fairly complex query requiring a dozen subgraph requests with some duplicate plans/requests. The goal here is to measure how well the gateways
behaves under a certain throughput.

K6 runs with a constant throughput of 1000 requests/s


## Latencies (ms)

![Latency Chart](charts/deduplication-latency.svg)

| Gateway                    |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :------------------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Hive Router                |    13.2 |    40.0 |    45.5 |    46.3 |    47.7 |    50.4 |
| Grafbase Gateway           |    19.4 |    41.4 |    46.0 |    47.0 |    48.3 |    55.7 |
| Cosmo Router               |    12.7 |    43.4 |    49.9 |    51.3 |    53.9 |    67.7 |
| Hive Gateway               |    81.1 |   234.5 |   348.9 |   389.3 |   440.6 |  1146.7 |
| Apollo Router (with dedup) |  errors |  errors |  errors |  errors |  errors |  errors |

## Resources

![Efficiency Chart](charts/deduplication-efficiency.svg)

| Gateway                    |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :------------------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Hive Router                |      80% ±2% |      84% |     188 ±5 MiB |   198 MiB |           1190.7 |         5164.9 |
| Grafbase Gateway           |     114% ±3% |     119% |      99 ±2 MiB |   103 MiB |            841.7 |         9960.6 |
| Hive Gateway               |    390% ±10% |     421% |  2516 ±238 MiB |  2687 MiB |            173.2 |          278.0 |
| Cosmo Router               |     569% ±6% |     582% |      84 ±5 MiB |    98 MiB |            171.6 |        10430.0 |
| Apollo Router (with dedup) |    974% ±13% |    1007% |     203 ±5 MiB |   218 MiB |           errors |         errors |

## Requests

![Quality Chart](charts/deduplication-quality.svg)

| Gateway                    | Requests | Failures | Subgraph requests (total) |
| :------------------------- | -------: | -------: | ------------------------: |
| Hive Gateway               |    43989 |        0 |              0.29 (12926) |
| Cosmo Router               |    60001 |        0 |              0.63 (37763) |
| Hive Router                |    60001 |        0 |              0.64 (38505) |
| Grafbase Gateway           |    60001 |        0 |              0.89 (53552) |
| Apollo Router (with dedup) |    60001 |       43 |              1.24 (74422) |

# long-lived-big-response

A very similar paylaod to big-response (~8MiB) is used, but now we add an extra subgraph request that takes 100ms. This forces the
gateway to keep the response for longer in memory and gives us a more realistic idea of how much cpu and memory a gateway would need.

K6 runs with 10 VUs to put some pressure on the gateways.


## Latencies (ms)

![Latency Chart](charts/long-lived-big-response-latency.svg)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Hive Router      |   107.6 |   141.7 |   155.5 |   160.2 |   168.2 |   266.2 |
| Grafbase Gateway |   128.2 |   158.3 |   175.4 |   181.7 |   194.8 |   277.3 |
| Cosmo Router     |   143.0 |   207.6 |   234.7 |   241.9 |   255.0 |   361.5 |
| Apollo Router    |   221.3 |   302.6 |   358.4 |   371.8 |   448.9 |   567.5 |
| Hive Gateway     |   210.8 |   399.3 |   569.2 |   592.4 |   751.6 |   792.2 |

## Resources

![Efficiency Chart](charts/long-lived-big-response-efficiency.svg)

| Gateway          |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :--------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Hive Router      |     226% ±8% |     246% |    520 ±15 MiB |   558 MiB |             28.1 |          126.4 |
| Grafbase Gateway |     215% ±7% |     230% |    323 ±20 MiB |   358 MiB |             26.8 |          176.5 |
| Hive Gateway     |     345% ±5% |     362% |   1759 ±55 MiB |  1852 MiB |              6.4 |           12.8 |
| Apollo Router    |    452% ±24% |     506% |  2285 ±183 MiB |  2678 MiB |              6.4 |           12.4 |
| Cosmo Router     |    811% ±96% |    1009% |    816 ±59 MiB |  1024 MiB |              4.7 |           47.7 |

## Requests

![Quality Chart](charts/long-lived-big-response-quality.svg)

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Hive Router      |     4145 |        0 |               0.36 (1510) |
| Apollo Router    |     1949 |        0 |               2.00 (3898) |
| Cosmo Router     |     2870 |        0 |               2.00 (5740) |
| Grafbase Gateway |     3711 |        0 |               2.00 (7422) |
| Hive Gateway     |     1399 |        0 |               2.00 (2798) |

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
| Grafbase Gateway            |     1.7 |     2.0 |     2.2 |     2.3 |     2.5 |    27.5 |
| Apollo Router               |     7.8 |     9.0 |     9.6 |     9.9 |    10.9 |  3327.7 |
| Cosmo Router                |    14.8 |    17.7 |    19.1 |    19.4 |    20.1 |   357.3 |
| Grafbase Gateway (no cache) |    18.4 |    19.5 |    20.2 |    20.5 |    22.0 |    27.4 |
| Cosmo Router (no cache)     |   339.8 |   351.6 |   356.0 |   357.5 |   361.6 |   372.3 |
| Apollo Router (no cache)    |  3279.4 |  3296.2 |  3308.8 |  3336.9 |  3384.1 |  3395.9 |
| Hive Gateway                |  errors |  errors |  errors |  errors |  errors |  errors |
| Hive Router                 |  errors |  errors |  errors |  errors |  errors |  errors |

## Resources

![Efficiency Chart](charts/many-plans-efficiency.svg)

| Gateway                     |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :-------------------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Grafbase Gateway            |     168% ±1% |     169% |      50 ±2 MiB |    53 MiB |            282.2 |         9245.7 |
| Apollo Router               |    172% ±15% |     176% |    469 ±47 MiB |   803 MiB |             58.6 |          131.6 |
| Grafbase Gateway (no cache) |     108% ±0% |     108% |     105 ±6 MiB |   125 MiB |             47.0 |          418.0 |
| Cosmo Router                |     508% ±6% |     523% |      73 ±2 MiB |    81 MiB |             10.6 |          703.5 |
| Cosmo Router (no cache)     |     160% ±6% |     169% |      78 ±4 MiB |    85 MiB |              1.7 |           34.5 |
| Apollo Router (no cache)    |     100% ±2% |     101% | 2187 ±1026 MiB |  4105 MiB |              0.3 |            0.1 |
| Hive Gateway                |     109% ±5% |     136% |  1303 ±117 MiB |  1344 MiB |           errors |         errors |
| Hive Router                 |     100% ±0% |     100% |     142 ±3 MiB |   148 MiB |           errors |         errors |

## Requests

![Quality Chart](charts/many-plans-quality.svg)

| Gateway                     | Requests | Failures | Subgraph requests (total) |
| :-------------------------- | -------: | -------: | ------------------------: |
| Grafbase Gateway            |    28675 |        0 |            77.7 (2227514) |
| Grafbase Gateway (no cache) |     3054 |        0 |             77.7 (237393) |
| Cosmo Router (no cache)     |      171 |        0 |               190 (32459) |
| Cosmo Router                |     3342 |        0 |              193 (645955) |
| Apollo Router               |     6192 |        0 |             203 (1256976) |
| Apollo Router (no cache)    |       19 |        0 |                203 (3857) |
| Hive Gateway                |     4817 |     4817 |              6.00 (28902) |
| Hive Router                 |        1 |        2 |                  0.00 (0) |

# query

Fairly complex query requiring a dozen subgraph requests with some duplicate plans/requests. The goal here is to measure how well the gateways
behaves under a certain throughput.

K6 runs with a constant throughput of 500 requests/s


## Latencies (ms)

![Latency Chart](charts/query-latency.svg)

| Gateway          |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :--------------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Hive Router      |    14.3 |    40.4 |    45.6 |    46.4 |    47.5 |    49.9 |
| Grafbase Gateway |    43.4 |    45.4 |    46.7 |    47.2 |    47.9 |    77.5 |
| Cosmo Router     |    44.0 |    46.9 |    48.3 |    48.8 |    49.7 |    56.3 |
| Apollo Router    |    45.4 |    48.0 |    49.0 |    49.6 |    50.5 |    76.1 |
| Hive Gateway     |    76.5 |   381.8 |   498.9 |   541.3 |   604.3 |   943.6 |

## Resources

![Efficiency Chart](charts/query-efficiency.svg)

| Gateway          |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :--------------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Hive Router      |      44% ±2% |      47% |     160 ±3 MiB |   169 MiB |           1066.4 |         3024.8 |
| Grafbase Gateway |      62% ±2% |      67% |      75 ±2 MiB |    79 MiB |            746.8 |         6443.9 |
| Apollo Router    |     263% ±4% |     275% |     182 ±5 MiB |   194 MiB |            181.9 |         2631.9 |
| Cosmo Router     |     334% ±4% |     346% |      62 ±2 MiB |    66 MiB |            144.2 |         7695.6 |
| Hive Gateway     |     378% ±9% |     406% |   2059 ±94 MiB |  2103 MiB |            114.4 |          226.2 |

## Requests

![Quality Chart](charts/query-quality.svg)

| Gateway          | Requests | Failures | Subgraph requests (total) |
| :--------------- | -------: | -------: | ------------------------: |
| Hive Router      |    29985 |        0 |              1.29 (38648) |
| Hive Gateway     |    28014 |        0 |             7.00 (196187) |
| Cosmo Router     |    29983 |        0 |             8.01 (240197) |
| Grafbase Gateway |    29985 |        0 |             13.0 (389805) |
| Apollo Router    |    29983 |        0 |             16.0 (479728) |

