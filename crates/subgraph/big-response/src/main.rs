use crate::schema::build_schema;

mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = build_schema();
    subgraph::serve(schema).await
}
