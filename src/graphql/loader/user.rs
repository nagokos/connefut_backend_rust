use async_graphql::dataloader::*;
use async_graphql::*;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::collections::HashMap;
use std::sync::Arc;

use crate::graphql::models::user::User;

pub struct UserLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<i64> for UserLoader {
    type Value = User;
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let sql = "SELECT * FROM users WHERE id IN (";
        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        let mut separated = query_builder.separated(", ");
        for key in keys.iter() {
            separated.push_bind(*key as i64);
        }
        separated.push_unseparated(") ");
        let query = query_builder.build_query_as::<User>();

        let users = query.fetch_all(&*self.pool).await?;

        // { user_id: User }の形に整形する
        let users_hash: HashMap<i64, User> = users.iter().map(|u| (u.id, u.to_owned())).collect();
        Ok(users_hash)
    }
}
