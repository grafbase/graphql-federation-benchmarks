use std::sync::{Arc, atomic::AtomicUsize};

use axum::{
    body::{Body, Bytes},
    http,
    response::{IntoResponse, Response},
};
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

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl std::ops::Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct AppStateInner {
    pub cache: Cache,
}

impl AppState {
    pub fn new() -> Self {
        let cache = Cache::with(
            1024,
            1024,
            UnitWeighter,
            Default::default(),
            Default::default(),
        );

        AppState(Arc::new(AppStateInner { cache }))
    }
}
