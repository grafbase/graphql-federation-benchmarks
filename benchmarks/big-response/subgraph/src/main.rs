use std::sync::atomic::AtomicUsize;

use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, SDLExportOptions, Schema,
};
use axum::{
    body::Body,
    extract::{Extension, Request},
    middleware::{self, Next},
    response::{self, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use schema::{create_schema, QueryRoot};

mod schema;

static COUNT: AtomicUsize = AtomicUsize::new(0);

type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(
    schema: Extension<AppSchema>,
    req: Json<async_graphql::Request>,
) -> impl IntoResponse {
    let response = schema.execute(req.0).await;
    Json(response)
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn count_requests(request: Request, next: Next) -> Response {
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

    // Skip health check requests only requesting '{"query":"{ __typename }"}' which is 26 bytes
    // Adding some extra margin in case gateways don't format it exactly like this. All benchmark
    // queries are bigger than that.
    if bytes.len() > 30 {
        COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    let request = Request::from_parts(parts, Body::from(bytes));
    next.run(request).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = create_schema();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route(
            "/sdl",
            get(move || async move { response::Html(sdl.clone()) }),
        )
        .route("/", get(graphiql))
        .route(
            "/stats",
            get(|| async {
                let count = COUNT.load(std::sync::atomic::Ordering::Relaxed);
                Json(serde_json::json!({"count": count}))
            }),
        )
        .layer(middleware::from_fn(count_requests))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7100").await?;
    println!("Listening on http://localhost:7100");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
