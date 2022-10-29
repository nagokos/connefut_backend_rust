use async_graphql::dataloader::*;
use async_graphql::*;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
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
            separated.push_bind(key);
        }
        separated.push_unseparated(") ");
        let query = query_builder.build_query_as::<User>();

        let users = query.fetch_all(&*self.pool).await?;

        // { user_id: User }の形に整形する
        let users_hash: HashMap<i64, User> = users.iter().map(|u| (u.id, u.to_owned())).collect();
        Ok(users_hash)
    }
}

pub struct FollowingLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<[i64; 2]> for FollowingLoader {
    type Value = ();
    type Error = Arc<sqlx::Error>;

    // keysは[follower_id, followed_id]の形で送られてくる
    async fn load(&self, keys: &[[i64; 2]]) -> Result<HashMap<[i64; 2], Self::Value>, Self::Error> {
        let sql = "SELECT follower_id, followed_id FROM relationships WHERE (follower_id, followed_id) IN";
        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        query_builder.push_tuples(keys, |mut b, key| {
            b.push_bind(key[0]).push_bind(key[1]);
        });
        let query = query_builder.build();
        let result = query.fetch_all(&*self.pool).await;
        match result {
            Ok(rows) => {
                tracing::info!("FollowingLoader load successed!!");
                // {[follower_id, followed_id], ()}の形に整形する
                let following_hash: HashMap<[i64; 2], ()> = rows
                    .iter()
                    .map(|row| {
                        let follower_id = row.get::<i64, _>("follower_id");
                        let followed_id = row.get::<i64, _>("followed_id");
                        ([follower_id, followed_id], ())
                    })
                    .collect();
                Ok(following_hash)
            }
            Err(e) => {
                tracing::error!("FollowingLoader load failed: {:?}", e);
                Err(e.into())
            }
        }
    }
}
