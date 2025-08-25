use std::sync::atomic::AtomicUsize;

use async_graphql::{ComplexObject, ID, Object, SimpleObject};

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Node {
    id0: String,
    id1: String,
    id2: String,
    id3: String,
    id4: String,
    id5: String,
    f0: Option<String>,
    f1: Option<String>,
    f2: Option<String>,
    f3: Option<String>,
    f4: Option<String>,
    f5: Option<String>,
    f6: Option<String>,
    f7: Option<String>,
    f8: Option<String>,
    f9: Option<String>,
}

#[ComplexObject]
impl Node {
    async fn n0(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n1(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n2(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n3(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n4(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n5(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n6(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n7(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n8(&self) -> Option<Node> {
        Some(Self::new())
    }
    async fn n9(&self) -> Option<Node> {
        Some(Self::new())
    }
}

impl Node {
    pub fn new() -> Self {
        #[allow(unused)]
        let id: u64 = rand::random();
        let id = 0;

        Node {
            id0: format!("id1-{id}"),
            id1: format!("id2-{id}"),
            id2: format!("id3-{id}"),
            id3: format!("id4-{id}"),
            id4: format!("id5-{id}"),
            id5: format!("id6-{id}"),
            f0: Some(format!("f0-{id}")),
            f1: Some(format!("f1-{id}")),
            f2: Some(format!("f2-{id}")),
            f3: Some(format!("f3-{id}")),
            f4: Some(format!("f4-{id}")),
            f5: Some(format!("f5-{id}")),
            f6: Some(format!("f6-{id}")),
            f7: Some(format!("f7-{id}")),
            f8: Some(format!("f8-{id}")),
            f9: Some(format!("f9-{id}")),
        }
    }
}

#[derive(Default)]
pub struct Query {}

#[Object]
impl Query {
    async fn node(&self) -> Option<Node> {
        sleep().await;
        Some(Node::new())
    }

    #[graphql(entity)]
    async fn find_node_by_id0(&self, id0: ID) -> Node {
        sleep().await;
        Node {
            id0: id0.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id1(&self, id1: ID) -> Node {
        sleep().await;
        Node {
            id1: id1.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id2(&self, id2: ID) -> Node {
        sleep().await;
        Node {
            id2: id2.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id3(&self, id3: ID) -> Node {
        sleep().await;
        Node {
            id3: id3.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id4(&self, id4: ID) -> Node {
        sleep().await;
        Node {
            id4: id4.to_string(),
            ..Node::new()
        }
    }

    #[graphql(entity)]
    async fn find_node_by_id5(&self, id5: ID) -> Node {
        sleep().await;
        Node {
            id5: id5.to_string(),
            ..Node::new()
        }
    }
}

static COUNT: AtomicUsize = AtomicUsize::new(0);

async fn sleep() {
    let current = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    println!("executing {current}");
    if let Some(delay) = std::env::var("DELAY_MS").ok().and_then(|v| v.parse().ok()) {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    }
}
