use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::graphql::models::sport::Sport;

pub struct SportLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<i64> for SportLoader {
    type Value = Sport;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let sql = "SELECT * FROM sports WHERE id IN (";
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(sql);
        let mut separated = query_builder.separated(", ");
        for key in keys.iter() {
            separated.push(key);
        }
        separated.push_unseparated(")");
        let query = query_builder.build_query_as::<Sport>();
        let sports = query.fetch_all(&*self.pool).await?;

        // { sport_id: Sport }の形になるように整形
        let sports_hash: HashMap<i64, Sport> = sports
            .iter()
            .map(|sport| (sport.id, sport.to_owned()))
            .collect();
        Ok(sports_hash)
    }
}
