use std::{ops::Range, sync::Arc};

use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish()
}

pub struct Query;

#[Object]
impl Query {
    async fn nodes(&self, _ctx: &Context<'_>, n: usize) -> Vec<Arc<Node>> {
        nodes(0..n)
    }

    #[graphql(entity)]
    async fn find_node_by_id(&self, id: usize) -> Option<Arc<Node>> {
        NODES.iter().find(|node| node.id == id).cloned()
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Node {
    id: usize,
    name: String,
    floats: Vec<f64>,
    ints: Vec<i32>,
    strings: Vec<String>,
    int: i32,
    float: f64,
    string: String,
}

#[ComplexObject]
impl Node {
    async fn nodes(&self, n: usize) -> Vec<Arc<Node>> {
        nodes((self.id * n)..((self.id + 1) * n))
    }
}

static NODES: Lazy<Vec<Arc<Node>>> = Lazy::new(|| {
    let mut rng = ChaCha8Rng::seed_from_u64(12345);

    (0..1000)
        .map(|id| {
            // Some numerical data
            let float_count = rng.random_range(0..=64);
            let int_count = rng.random_range(0..=64);
            let string_count = rng.random_range(0..=64);

            let floats: Vec<f64> = (0..float_count)
                .map(|_| rng.random_range(-1000.0..1000.0))
                .collect();

            let ints: Vec<i32> = (0..int_count)
                .map(|_| rng.random_range(-10000..10000))
                .collect();

            // Most strings tend to be simple text that doesn't need escaping.
            let strings: Vec<String> = (0..string_count)
                .map(|_| {
                    let len = rng.random_range(0..=128);
                    (0..len)
                        .map(|_| {
                            let c = rng.random_range(b'a'..=b'z');
                            c as char
                        })
                        .collect()
                })
                .collect();

            // But we still want some that requires escaping.
            let string_len = rng.random_range(10..=128);
            let string: String = (0..string_len)
                .map(|_| {
                    let c = rng.random_range(b'!'..=b'}');
                    c as char
                })
                .collect();

            Arc::new(Node {
                id,
                name: id.to_string(),
                floats,
                ints,
                strings,
                int: rng.random_range(-10000..10000),
                float: rng.random_range(-1000.0..1000.0),
                string,
            })
        })
        .collect()
});

fn nodes(range: Range<usize>) -> Vec<Arc<Node>> {
    let len = NODES.len();
    let count = range.end - range.start;

    (0..count)
        .map(|i| {
            let index = (range.start + i) % len;
            NODES[index].clone()
        })
        .collect()
}
