use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::{self, IntoResponse, Response},
    routing::{get, post_service},
};
use schema::Query;
use tokio::net::TcpListener;

mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql", post_service(GraphQL::new(schema)))
        .route("/sdl", get(|| async move { response::Html(sdl.clone()) }))
        .route("/", get(graphiql))
        .layer(middleware::from_fn(print_request_body));

    println!("GraphiQL IDE: http://localhost:7000");
    axum::serve(TcpListener::bind("127.0.0.1:7000").await?, app).await?;

    Ok(())
}

async fn print_request_body(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();

    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("Failed to read request body: {err}");
            return Response::builder()
                .status(400)
                .body(Body::from("Failed to read request body"))
                .unwrap();
        }
    };

    if !bytes.is_empty() {
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        println!("Request body: {}", serde_json::to_string_pretty(&value).unwrap());
    }

    let request = Request::from_parts(parts, Body::from(bytes));
    next.run(request).await
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
