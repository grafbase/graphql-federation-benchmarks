// It's unclear to me whether this no cache really works like I expect it to.
// Today Hive fails on the many-plans case and every time I make a request to Hive, latencies go down.
// I understand it to be the JS JIT doing its work, but hard to verify compared to AOT compiled languages like Go and Rust.
import { defineConfig } from "@graphql-hive/gateway";

export const gatewayConfig = defineConfig({
  parserAndValidationCache: false,
  cache: {
    get: async () => undefined,
    set: async () => {},
    delete: async () => {},
  },
  propagateHeaders: {
    fromClientToSubgraphs({ request, subgraphName }) {
      return {
        Authorization: request.headers.get("Authorization"),
      };
    },
  },
});
