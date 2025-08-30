mod state;

use std::{sync::atomic::Ordering, time::Duration};

use async_graphql::{SDLExportOptions, Schema, http::GraphiQLSource};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Extension, Path, State},
    http,
    response::{self, IntoResponse},
    routing::{get, post},
};

use crate::state::{AppState, CachedResponse};

pub async fn serve<Query, Mutation, Subscription>(
    schema: Schema<Query, Mutation, Subscription>,
) -> anyhow::Result<()>
where
    Query: async_graphql::ObjectType + 'static,
    Mutation: async_graphql::ObjectType + 'static,
    Subscription: async_graphql::SubscriptionType + 'static,
{
    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql/{subgraph_name}", post(graphql_handler::<Query, Mutation, Subscription>))
        .with_state(AppState::new())
        .route("/healthcheck", get(|| async { "OK" }))
        .route(
            "/sdl",
            get(move || async move { response::Html(sdl.clone()) }),
        )
        .route("/", get(graphiql))
        .route(
            "/stats",
            get(|| async {
                let count = state::COUNT.load(Ordering::Relaxed);
                let cache_hit = state::CACHE_HIT.load(Ordering::Relaxed);
                let cache_miss = state::CACHE_MISS.load(Ordering::Relaxed);
                Json(serde_json::json!({"count": count, "cache_hit": cache_hit, "cache_miss": cache_miss}))
            }),
        )
        .layer(Extension(schema));

    let port = std::env::var("PORT").unwrap_or_else(|_| "7471".to_string());

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Listening on http://localhost:{}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn graphql_handler<Query, Mutation, Subscription>(
    State(state): State<AppState>,
    Path((subgraph_name,)): Path<(String,)>,
    schema: Extension<Schema<Query, Mutation, Subscription>>,
    bytes: Bytes,
) -> impl IntoResponse
where
    Query: async_graphql::ObjectType + 'static,
    Mutation: async_graphql::ObjectType + 'static,
    Subscription: async_graphql::SubscriptionType + 'static,
{
    // Skip health check requests only requesting '{"query":"{ __typename }"}' which is 26 bytes
    // Adding some extra margin in case gateways don't format it exactly like this. All benchmark
    // queries are bigger than that.
    if bytes.len() > 30 {
        state::COUNT.fetch_add(1, Ordering::Relaxed);
    }

    if let Some(delay_ms) = std::env::var(format!("{}_DELAY_MS", subgraph_name.to_uppercase()))
        .ok()
        .and_then(|ms| ms.parse::<u64>().ok())
        .filter(|&ms| ms > 0)
    {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    match state.cache.get_value_or_guard_async(&bytes).await {
        Ok(response) => {
            state::CACHE_HIT.fetch_add(1, Ordering::Relaxed);
            response
        }
        Err(guard) => {
            state::CACHE_MISS.fetch_add(1, Ordering::Relaxed);
            let request: async_graphql::Request = serde_json::from_slice(&bytes).unwrap();
            let response = schema.execute(request).await;
            let bytes = Bytes::from(serde_json::to_vec(&response).unwrap());
            let response = CachedResponse {
                status: http::StatusCode::OK,
                headers: {
                    let mut headers = http::HeaderMap::new();
                    headers.insert(
                        http::header::CONTENT_TYPE,
                        http::HeaderValue::from_static("application/json"),
                    );
                    headers.insert(
                        http::header::CONTENT_LENGTH,
                        http::HeaderValue::from_str(&bytes.len().to_string()).unwrap(),
                    );
                    headers
                },
                body: bytes.clone(),
            };
            let _ = guard.insert(response.clone());
            response
        }
    }
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
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
