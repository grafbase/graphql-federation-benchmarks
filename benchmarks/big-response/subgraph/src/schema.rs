use std::{ops::Range, sync::Arc};

use async_graphql::{
    ComplexObject, Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Node {
    id: usize,
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
            let float_count = rng.gen_range(0..=16);
            let int_count = rng.gen_range(0..=32);
            let string_count = rng.gen_range(0..=8);

            let floats: Vec<f64> = (0..float_count)
                .map(|_| rng.gen_range(-1000.0..1000.0))
                .collect();

            let ints: Vec<i32> = (0..int_count)
                .map(|_| rng.gen_range(-10000..10000))
                .collect();

            let strings: Vec<String> = (0..string_count)
                .map(|_| {
                    let len = rng.gen_range(0..=256);
                    (0..len)
                        .map(|_| {
                            let c = rng.gen_range(b'a'..=b'z');
                            c as char
                        })
                        .collect()
                })
                .collect();

            let string_len = rng.gen_range(10..=100);
            let string: String = (0..string_len)
                .map(|_| {
                    let c = rng.gen_range(b'A'..=b'Z');
                    c as char
                })
                .collect();

            Arc::new(Node {
                id,
                floats,
                ints,
                strings,
                int: rng.gen_range(-10000..10000),
                float: rng.gen_range(-1000.0..1000.0),
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

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn nodes(&self, _ctx: &Context<'_>, n: usize) -> Vec<Arc<Node>> {
        nodes(0..n)
    }
}

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish()
}
