use async_graphql::*;

pub mod prefecture_resolver;
pub mod sport_resolver;
pub mod tag_resolver;
pub mod user_resolver;

use crate::graphql::models::{prefecture::Prefecture, sport::Sport, tag::Tag, user::User};

#[derive(Interface)]
#[graphql(field(name = "id", type = "ID"))]
pub enum Node {
    Prefecture(Prefecture),
    Sport(Sport),
    Tag(Tag),
    User(User),
}

#[derive(Default)]
pub struct RootQuery;

#[Object]
impl RootQuery {
    async fn node(&self, _ctx: &Context<'_>, _id: ID) -> Option<Node> {
        let tag = Tag {
            id: 9,
            name: String::from("tag"),
        };
        Some(Node::Tag(tag))
    }
}
