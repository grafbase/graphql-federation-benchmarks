//! Copied and adjust from The Guild's GraphQL Gateways Benchmark
//! https://github.com/graphql-hive/graphql-gateways-benchmark
use async_graphql::{EmptyMutation, EmptySubscription, ID, Object, Schema, SimpleObject};
use once_cell::sync::Lazy;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish()
}

static USERS: Lazy<Vec<User>> = Lazy::new(|| {
    vec![
        User {
            id: ID("1".to_string()),
            name: Some("Uri Goldshtein".to_string()),
            username: Some("urigo".to_string()),
            birthday: Some(1234567890),
        },
        User {
            id: ID("2".to_string()),
            name: Some("Dotan Simha".to_string()),
            username: Some("dotansimha".to_string()),
            birthday: Some(1234567890),
        },
        User {
            id: ID("3".to_string()),
            name: Some("Kamil Kisiela".to_string()),
            username: Some("kamilkisiela".to_string()),
            birthday: Some(1234567890),
        },
        User {
            id: ID("4".to_string()),
            name: Some("Arda Tanrikulu".to_string()),
            username: Some("ardatan".to_string()),
            birthday: Some(1234567890),
        },
        User {
            id: ID("5".to_string()),
            name: Some("Gil Gardosh".to_string()),
            username: Some("gilgardosh".to_string()),
            birthday: Some(1234567890),
        },
        User {
            id: ID("6".to_string()),
            name: Some("Laurin Quast".to_string()),
            username: Some("laurin".to_string()),
            birthday: Some(1234567890),
        },
    ]
});

#[derive(SimpleObject, Clone)]
struct User {
    id: ID,
    name: Option<String>,
    username: Option<String>,
    birthday: Option<i32>,
}

impl User {
    fn me() -> User {
        USERS[0].clone()
    }
}

pub struct Query;

#[Object(extends = true)]
impl Query {
    async fn me(&self) -> Option<User> {
        Some(User::me())
    }

    async fn user(&self, id: ID) -> Option<User> {
        USERS.iter().find(|user| user.id == id).cloned()
    }

    async fn users(&self) -> Option<Vec<Option<User>>> {
        Some(USERS.iter().map(|user| Some(user.clone())).collect())
    }

    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        USERS.iter().find(|user| user.id == id).cloned().unwrap()
    }
}
