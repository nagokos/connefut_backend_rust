use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::graphql::models::prefecture::Prefecture;

pub struct PrefectureLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<i64> for PrefectureLoader {
    type Value = Prefecture;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let sql = "SELECT * FROM prefectures WHERE id IN (";
        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        let mut separated = query_builder.separated(", ");
        for key in keys.iter() {
            separated.push_bind(key);
        }
        separated.push_unseparated(") ");
        let query = query_builder.build_query_as::<Prefecture>();
        let prefectures = query.fetch_all(&*self.pool).await?;

        // { prefecture_id: Prefecture }の形になるように整形
        let prefectures_hash: HashMap<i64, Prefecture> = prefectures
            .iter()
            .map(|prefecture| (prefecture.id, prefecture.to_owned()))
            .collect();
        Ok(prefectures_hash)
    }
}
