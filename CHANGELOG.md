## Changelog

### 2025-09-23

Change the `many-plans` scenario to have different ids for every node to avoid the influence of in-flight request de-duplication or something else? Not sure, but it had much more impact that expected. Hive Gateway went from \~40 to 85 subgraph requests and Cosmo Router from \~193 to 380 subgraph requests. Grafbase Gateway went from \~78 to 83 subgraph requests and Apollo Router didn't change.
