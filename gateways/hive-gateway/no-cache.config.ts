// It's unclear to me whether this no cache really works like I expect it to.
// Today Hive fails on the many-plans case and every time I make a request to Hive, latencies go down.
// I understand it to be the JS JIT doing its work, but hard to verify compared to AOT compiled languages like Go and Rust.
import { defineConfig } from "@graphql-hive/gateway";
import { HTTPTransportOptions } from "@graphql-tools/executor-http";

export const gatewayConfig = defineConfig({
  parserAndValidationCache: false,
  cache: {
    get: async () => undefined,
    set: async () => { },
    delete: async () => { },
  },
  transportEntries: {
    "*.http": {
      options: {
        deduplicateInflightRequests: false,
      } as HTTPTransportOptions,
    },
  },
  propagateHeaders: {
    fromClientToSubgraphs({ request, subgraphName }) {
      return {
        Authorization: request.headers.get("Authorization"),
      };
    },
  },
});
