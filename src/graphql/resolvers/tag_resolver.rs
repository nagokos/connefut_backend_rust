use async_graphql::{Context, Object, Result};

use crate::{
    database::get_db_pool,
    graphql::models::tag::{get_tags, Tag},
};

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
