use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

pub struct StockLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<i64> for StockLoader {
    type Value = i64;
    type Error = Arc<sqlx::Error>;

    // 募集がストックされている数
    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let sql = "SELECT recruitment_id, COUNT(DISTINCT id) FROM stocks WHERE recruitment_id IN (";
        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        let mut separated = query_builder.separated(",");
        for key in keys.iter() {
            separated.push_bind(key);
        }
        separated.push_unseparated(") ");
        query_builder.push("GROUP BY recruitment_id");
        let query = query_builder.build();
        let result = query.fetch_all(&*self.pool).await;

        match result {
            Ok(rows) => {
                tracing::info!("StockLoader load stocked_count successed!!");
                let stock_hash: HashMap<i64, i64> = rows
                    .iter()
                    .map(|row| {
                        let recruitment_id: i64 = row.get("recruitment_id");
                        let count: i64 = row.get("count");
                        (recruitment_id, count)
                    })
                    .collect();
                Ok(stock_hash)
            }
            Err(e) => {
                tracing::error!("StockLoader load stocked_count failed: {:?}", e);
                Err(e.into())
            }
        }
    }
}

#[async_trait]
impl Loader<[i64; 2]> for StockLoader {
    type Value = ();
    type Error = Arc<sqlx::Error>;

    // keysは[user_id, recruitment_id]の形で送られてくる
    // ユーザーが募集をストックしているか
    async fn load(&self, keys: &[[i64; 2]]) -> Result<HashMap<[i64; 2], Self::Value>, Self::Error> {
        let sql = "SELECT user_id, recruitment_id FROM stocks WHERE (user_id, recruitment_id) IN";
        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        query_builder.push_tuples(keys, |mut b, key| {
            b.push_bind(key[0]).push_bind(key[1]);
        });
        let query = query_builder.build();
        let result = query.fetch_all(&*self.pool).await;
        match result {
            Ok(rows) => {
                tracing::info!("RecruitmentLoader load viewer_has_stocked successed!!");
                // {[user_id, recruitment_id], ()}の形に整形する
                let recruitment_hash: HashMap<[i64; 2], ()> = rows
                    .iter()
                    .map(|row| {
                        let user_id: i64 = row.get("user_id");
                        let recruitment_id: i64 = row.get("recruitment_id");
                        ([user_id, recruitment_id], ())
                    })
                    .collect();
                Ok(recruitment_hash)
            }
            Err(e) => {
                tracing::error!("RecruitmentLoader load viewer_has_stocked failed: {:?}", e);
                Err(e.into())
            }
        }
    }
}
