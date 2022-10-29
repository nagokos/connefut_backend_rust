use async_graphql::{Context, Object, Result};

use crate::database::get_db_pool;
use crate::graphql::models::sport::{get_sports, Sport};

#[derive(Default)]
pub struct SportQuery;

#[Object]
impl SportQuery {
    /// スポーツのリストを取得する
    async fn sports(&self, ctx: &Context<'_>) -> Result<Vec<Sport>> {
        let pool = get_db_pool(ctx).await?;
        let sports = get_sports(pool).await?;
        Ok(sports)
    }
}
