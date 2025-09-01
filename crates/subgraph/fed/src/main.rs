mod schema;

use schema::{
    build_accounts_schema, build_inventory_schema, build_products_schema, build_reviews_schema,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = subgraph::AppState::builder()
        .with_subgraph("accounts", Box::new(build_accounts_schema()))
        .with_subgraph("inventory", Box::new(build_inventory_schema()))
        .with_subgraph("products", Box::new(build_products_schema()))
        .with_subgraph("reviews", Box::new(build_reviews_schema()))
        .build();

    subgraph::serve(state).await
}
