use async_graphql::*;

use crate::database::get_db_pool;
use crate::graphql::models::prefecture::{get_prefectures, Prefecture};

#[derive(Default)]
pub struct PrefectureQuery;

#[Object]
impl PrefectureQuery {
    /// 都道府県のリストを取得する
    async fn prefectures(&self, ctx: &Context<'_>) -> Result<Vec<Prefecture>> {
        let pool = get_db_pool(ctx).await?;
        let prefectures = get_prefectures(pool).await?;
        Ok(prefectures)
    }
}
