import { defineConfig } from "@graphql-hive/gateway";

// Doesn't work, no idea how to disable hive plan cache whether in config or code.
export const gatewayConfig = defineConfig({
  parserAndValidationCache: false,
  cache: {
    get: async () => undefined,
    set: async () => { },
    delete: async () => { },
  },
});
