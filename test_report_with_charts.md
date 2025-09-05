# System Information

Date: 2025-01-05
CPU: Intel Core i9
Memory: 32.0 GiB
CPU Boost: Enabled
Git Commit: abc123def456
Linux Version: 6.16.1
Docker Version: 24.0.7

# Benchmarks

## big-response

Test scenario for large GraphQL responses

### Requests

![Quality Chart](charts/big-response-quality.svg)

| Gateway      | Requests | Failures | Subgraph requests (total) |
| :----------- | -------: | -------: | ------------------------: |
| Grafbase     |      100 |        0 |                2.00 (200) |
| Apollo       |       95 |        0 |                2.10 (200) |
| Cosmo        |       98 |        0 |                2.04 (200) |

### Latencies (ms)

![Latency Chart](charts/big-response-latency.svg)

| Gateway      |     Min |     Med |     P90 |     P95 |     P99 |     Max |
| :----------- | ------: | ------: | ------: | ------: | ------: | ------: |
| Grafbase     |    15.2 |    20.1 |    25.3 |    28.4 |    32.7 |    45.6 |
| Apollo       |    18.5 |    24.3 |    30.2 |    33.5 |    38.9 |    52.1 |
| Cosmo        |    16.8 |    22.0 |    27.5 |    30.8 |    35.2 |    48.3 |

### Resources

![Efficiency Chart](charts/big-response-efficiency.svg)

| Gateway      |          CPU |  CPU max |         Memory |   MEM max |  requests/core.s |  requests/GB.s |
| :----------- | -----------: | -------: | -------------: | --------: | ---------------: | -------------: |
| Grafbase     |       5% ±2% |      12% |     256 ±15 MiB |   300 MiB |            416.7 |          333.3 |
| Apollo       |       7% ±3% |      18% |     512 ±25 MiB |   600 MiB |            263.9 |          158.3 |
| Cosmo        |       6% ±2% |      15% |     384 ±20 MiB |   450 MiB |            326.7 |          217.8 |