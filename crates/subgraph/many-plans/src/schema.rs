use async_graphql::{
    ComplexObject, EmptyMutation, EmptySubscription, ID, Object, Schema, SimpleObject,
};

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish()
}

pub struct Query;

#[Object]
impl Query {
    async fn node(&self) -> Option<Node> {
        Some(Node::new("0".to_string()))
    }

    #[graphql(entity)]
    async fn find_node_by_id0(&self, id0: ID) -> Node {
        Node {
            id0: id0.to_string(),
            ..Node::new(id0.into())
        }
    }
}

#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct Node {
    id0: String,
    f0: Option<String>,
    f1: Option<String>,
    f2: Option<String>,
    f3: Option<String>,
    f4: Option<String>,
    f5: Option<String>,
    f6: Option<String>,
}

#[ComplexObject]
impl Node {
    async fn n0(&self) -> Option<Node> {
        Some(Self::new(format!("{}0", self.id0)))
    }
    async fn n1(&self) -> Option<Node> {
        Some(Self::new(format!("{}1", self.id0)))
    }
    async fn n2(&self) -> Option<Node> {
        Some(Self::new(format!("{}2", self.id0)))
    }
    async fn n3(&self) -> Option<Node> {
        Some(Self::new(format!("{}3", self.id0)))
    }
    async fn n4(&self) -> Option<Node> {
        Some(Self::new(format!("{}4", self.id0)))
    }
    async fn n5(&self) -> Option<Node> {
        Some(Self::new(format!("{}5", self.id0)))
    }
    async fn n6(&self) -> Option<Node> {
        Some(Self::new(format!("{}6", self.id0)))
    }
}

impl Node {
    pub fn new(id0: String) -> Self {
        Node {
            f0: Some(format!("f0-{id0}")),
            f1: Some(format!("f1-{id0}")),
            f2: Some(format!("f2-{id0}")),
            f3: Some(format!("f3-{id0}")),
            f4: Some(format!("f4-{id0}")),
            f5: Some(format!("f5-{id0}")),
            f6: Some(format!("f6-{id0}")),
            id0,
        }
    }
}
