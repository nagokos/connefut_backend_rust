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

#[derive(Debug)]
pub struct PageInfo {
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[Object]
impl PageInfo {
    async fn start_cursor(&self) -> Option<&str> {
        self.start_cursor.as_deref()
    }
    async fn end_cursor(&self) -> Option<&str> {
        self.end_cursor.as_deref()
    }
    async fn has_next_page(&self) -> bool {
        self.has_next_page
    }
    async fn has_previous_page(&self) -> bool {
        self.has_previous_page
    }
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
