use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

use async_graphql::{Request, Response as GraphQLResponse, SDLExportOptions, Schema};
use axum::{
    body::{Body, Bytes},
    http,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use quick_cache::UnitWeighter;

pub static COUNT: AtomicUsize = AtomicUsize::new(0);
pub static CACHE_HIT: AtomicUsize = AtomicUsize::new(0);
pub static CACHE_MISS: AtomicUsize = AtomicUsize::new(0);

type Cache = quick_cache::sync::Cache<
    Bytes,
    CachedResponse,
    UnitWeighter,
    rapidhash::fast::SeedableState<'static>,
>;

#[derive(Clone)]
pub struct CachedResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: Bytes,
}

impl IntoResponse for CachedResponse {
    fn into_response(self) -> Response {
        let mut response = Response::builder().status(self.status);
        *response.headers_mut().unwrap() = self.headers;
        response.body(Body::from(self.body)).unwrap()
    }
}

pub trait BoxedSchema: Send + Sync {
    fn execute(&self, request: Request) -> BoxFuture<'_, GraphQLResponse>;
    fn sdl(&self) -> String;
}

impl<Query, Mutation, Subscription> BoxedSchema
    for async_graphql::Schema<Query, Mutation, Subscription>
where
    Query: async_graphql::ObjectType + 'static,
    Mutation: async_graphql::ObjectType + 'static,
    Subscription: async_graphql::SubscriptionType + 'static,
{
    fn execute(&self, request: Request) -> BoxFuture<'_, GraphQLResponse> {
        Box::pin(async_graphql::Schema::execute(self, request))
    }

    fn sdl(&self) -> String {
        self.sdl_with_options(SDLExportOptions::new().federation().compose_directive())
    }
}

pub struct SubgraphState {
    pub cache: Cache,
    pub schema: Box<dyn BoxedSchema>,
}

pub enum AppState {
    Single(SubgraphState),
    PerSubgraph(HashMap<String, SubgraphState>),
}

impl SubgraphState {
    fn new(schema: Box<dyn BoxedSchema>) -> Self {
        let cache = Cache::with(
            1024,
            1024,
            UnitWeighter,
            Default::default(),
            Default::default(),
        );
        SubgraphState { cache, schema }
    }
}

impl AppState {
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::new()
    }

    pub fn single(schema: Box<dyn BoxedSchema>) -> Self {
        AppState::Single(SubgraphState::new(schema))
    }

    pub fn get_subgraph(&self, name: &str) -> Option<&SubgraphState> {
        match self {
            AppState::Single(state) => Some(state),
            AppState::PerSubgraph(map) => map.get(name),
        }
    }
}

pub struct AppStateBuilder {
    schemas: HashMap<String, SubgraphState>,
}

impl Default for AppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStateBuilder {
    pub fn new() -> Self {
        AppStateBuilder {
            schemas: HashMap::new(),
        }
    }

    pub fn with_subgraph(mut self, name: impl Into<String>, schema: Box<dyn BoxedSchema>) -> Self {
        self.schemas.insert(name.into(), SubgraphState::new(schema));
        self
    }

    pub fn build(self) -> AppState {
        if self.schemas.is_empty() {
            panic!("AppStateBuilder requires at least one subgraph")
        }
        AppState::PerSubgraph(self.schemas)
    }
}

impl<Query, Mutation, Subscription> From<Schema<Query, Mutation, Subscription>> for AppState
where
    Query: async_graphql::ObjectType + 'static,
    Mutation: async_graphql::ObjectType + 'static,
    Subscription: async_graphql::SubscriptionType + 'static,
{
    fn from(schema: Schema<Query, Mutation, Subscription>) -> Self {
        AppState::single(Box::new(schema))
    }
}
