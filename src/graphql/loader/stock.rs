use std::{collections::HashMap, sync::Arc};

use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

pub struct StockLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<[i64; 2]> for StockLoader {
    type Value = ();
    type Error = Arc<sqlx::Error>;

    async fn load(&self, keys: &[[i64; 2]]) -> Result<HashMap<[i64; 2], Self::Value>, Self::Error> {
        println!("{:?}", keys);
        let sql = "SELECT * FROM stocks WHERE (user_id, recruitment_id) IN";
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(sql);
        query_builder.push_tuples(keys, |mut b, key| {
            b.push_bind(key[0]).push_bind(key[1]);
        });
        let query = query_builder.build();
        let rows = query.fetch_all(&*self.pool).await?;
        // { [user_id, recruitment_id]: () }の形に整形する
        // load_oneはOption型を返すので組み合わせがあればviwerHasStockedをtrueにする
        let stock_hash: HashMap<[i64; 2], ()> = rows
            .iter()
            .map(|row| {
                let user_id: i64 = row.get("user_id");
                let recruitment_id: i64 = row.get("recruitment_id");
                ([user_id, recruitment_id], ())
            })
            .collect();
        Ok(stock_hash)
    }
}
