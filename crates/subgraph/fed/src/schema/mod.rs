//! Copied and adjust from The Guild's GraphQL Gateways Benchmark
//! https://github.com/graphql-hive/graphql-gateways-benchmark
mod accounts;
mod inventory;
mod products;
mod reviews;

pub use accounts::build_schema as build_accounts_schema;
pub use inventory::build_schema as build_inventory_schema;
pub use products::build_schema as build_products_schema;
pub use reviews::build_schema as build_reviews_schema;
