mod state;

use serde::Serialize;
pub use state::{AppState, AppStateBuilder};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{sync::atomic::Ordering, time::Duration};

use async_graphql::http::GraphiQLSource;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Request, State},
    http,
    middleware::{self, Next},
    response::{self, IntoResponse},
    routing::{get, post},
};

use std::sync::Arc;

use crate::state::CachedResponse;

pub async fn serve(state: impl Into<AppState>) -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(std::env::var("RUST_LOG").is_ok()))
        .init();

    let state = Arc::new(state.into());
    let app = Router::new()
        .route("/graphql/{subgraph_name}", post(graphql_handler))
        .route("/sdl", get(sdl_handler))
        .route("/sdl/{subgraph_name}", get(sdl_subgraph_handler))
        .with_state(state)
        .route("/healthcheck", get(|| async { "OK" }))
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
        .layer(middleware::from_fn(request_logging_middleware));

    let port = std::env::var("PORT").unwrap_or_else(|_| "7471".to_string());

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("Listening on http://localhost:{}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn request_logging_middleware(req: Request, next: Next) -> impl IntoResponse {
    debug!("HTTP {} {}", req.method(), req.uri());
    next.run(req).await
}

async fn graphql_handler(
    State(state): State<Arc<AppState>>,
    Path((subgraph_name,)): Path<(String,)>,
    #[allow(unused_variables)] headers: http::HeaderMap,
    bytes: Bytes,
) -> impl IntoResponse {
    // TODO: Hive-router doesn't forward headers....
    // Check for Authorization header
    // if !headers.contains_key(http::header::AUTHORIZATION) {
    //     return response::Response::builder()
    //         .status(http::StatusCode::BAD_REQUEST)
    //         .body(axum::body::Body::from(
    //             r#"{"errors": [{"message":"Missing Authorization header"}]}"#,
    //         ))
    //         .unwrap()
    //         .into_response();
    // }

    // Get the appropriate subgraph state
    let subgraph = match state.get_subgraph(&subgraph_name) {
        Some(s) => s,
        None => {
            return response::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::from(format!(
                    "Subgraph '{}' not found",
                    subgraph_name
                )))
                .unwrap()
                .into_response();
        }
    };

    // Skip health check requests only requesting '{"query":"{ __typename }"}' which is 26 bytes
    // Adding some extra margin in case gateways don't format it exactly like this. All benchmark
    // queries are bigger than that.
    if bytes.len() > 30 {
        state::COUNT.fetch_add(1, Ordering::Relaxed);
    }

    // Check for subgraph-specific delay first, then fall back to general DELAY_MS
    let delay_ms = std::env::var(format!("{}_DELAY_MS", subgraph_name.to_uppercase()))
        .or_else(|_| std::env::var("DELAY_MS"))
        .ok()
        .and_then(|ms| ms.parse::<u64>().ok());

    if let Some(delay_ms) = delay_ms.filter(|&ms| ms > 0) {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    debug!(subgraph = %subgraph_name, request_body = %String::from_utf8_lossy(bytes.as_ref()), "Received GraphQL request");

    match subgraph.cache.get_value_or_guard_async(&bytes).await {
        Ok(response) => {
            state::CACHE_HIT.fetch_add(1, Ordering::Relaxed);
            response.into_response()
        }
        Err(guard) => {
            state::CACHE_MISS.fetch_add(1, Ordering::Relaxed);
            let request: async_graphql::Request = serde_json::from_slice(&bytes).unwrap();
            let response = subgraph.schema.execute(request).await;
            let body = Bytes::from(serde_json::to_vec(&response).unwrap());
            debug!(subgraph = %subgraph_name, response_body = %String::from_utf8_lossy(body.as_ref()), "Sending GraphQL response");

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
                        http::HeaderValue::from_str(&body.len().to_string()).unwrap(),
                    );
                    headers
                },
                body,
            };
            let _ = guard.insert(response.clone());
            response.into_response()
        }
    }
}

async fn sdl_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.as_ref() {
        AppState::Single(subgraph) => response::Html(subgraph.schema.sdl()).into_response(),
        AppState::PerSubgraph(map) => {
            // Return SDLs for all subgraphs as JSON
            let sdls: std::collections::HashMap<_, _> = map
                .iter()
                .map(|(name, subgraph)| (name.clone(), subgraph.schema.sdl()))
                .collect();
            Json(sdls).into_response()
        }
    }
}

async fn sdl_subgraph_handler(
    State(state): State<Arc<AppState>>,
    Path((subgraph_name,)): Path<(String,)>,
) -> impl IntoResponse {
    match state.get_subgraph(&subgraph_name) {
        Some(subgraph) => response::Html(subgraph.schema.sdl()).into_response(),
        None => response::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::from(format!(
                "Subgraph '{}' not found",
                subgraph_name
            )))
            .unwrap()
            .into_response(),
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

/// Serialize a type implementing [`serde::Serialize`] and return the encoded byte vector.
/// Please use this instead of `minicbor_serde::to_vec` due to this function serializing nulls correctly.
pub fn to_cbor_vec<T: Serialize>(
    val: T,
) -> Result<Vec<u8>, minicbor_serde::error::EncodeError<core::convert::Infallible>> {
    let mut serialized = Vec::new();
    let mut serializer = minicbor_serde::Serializer::new(&mut serialized);

    // Necessary for serde_json::Value which serializes `Null` as unit rather than none...
    serializer.serialize_unit_as_null(true);
    val.serialize(&mut serializer)?;

    Ok(serialized)
}
