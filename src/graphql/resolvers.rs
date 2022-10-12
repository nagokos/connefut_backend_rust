pub use async_graphql::*;

pub mod prefecture_resolver;
pub mod recruitment_resolver;
pub mod sport_resolver;
pub mod tag_resolver;
pub mod user_resolver;

use crate::graphql::models::{
    prefecture::Prefecture, recruitment::Recruitment, sport::Sport, tag::Tag, user::User,
};

//* Node interface */
#[derive(Interface)]
#[graphql(field(name = "id", type = "ID"))]
pub enum Node {
    Prefecture(Prefecture),
    Sport(Sport),
    Tag(Tag),
    User(User),
    Recruitment(Recruitment),
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
