use async_graphql::{Context, Object, Result};
use base64::{encode_config, URL_SAFE};

use crate::{
    database::get_db_pool,
    graphql::{
        models::tag::{self, get_tags, Tag},
        mutations::tag_mutation::{CreateTagInput, CreateTagResult, CreateTagSuccess},
    },
};

#[derive(Debug)]
pub struct TagEdge {
    pub cursor: String,
    pub node: Tag,
}

#[Object]
impl TagEdge {
    pub async fn cursor(&self) -> String {
        encode_config(format!("Tag:{}", self.node.id), URL_SAFE)
    }
    pub async fn node(&self) -> Tag {
        self.node.clone()
    }
}

#[derive(Default)]
pub struct TagQuery;

#[Object]
impl TagQuery {
    async fn tags(&self, ctx: &Context<'_>) -> Result<Vec<Tag>> {
        let pool = get_db_pool(ctx).await?;
        let tags = get_tags(pool).await?;
        Ok(tags)
    }
}

#[derive(Default)]
pub struct TagMutation;

#[Object]
impl TagMutation {
    async fn create_tag(
        &self,
        ctx: &Context<'_>,
        input: CreateTagInput,
    ) -> Result<CreateTagResult> {
        let pool = get_db_pool(ctx).await?;

        if let Some(error) = input.validate_is_already_tag_name(pool).await? {
            return Ok(error.into());
        }

        let tag = tag::create(pool, &input.name).await?;
        let tag_edge = TagEdge {
            cursor: String::default(),
            node: tag,
        };
        Ok(CreateTagSuccess { tag_edge }.into())
    }
}
