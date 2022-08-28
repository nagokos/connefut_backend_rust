use async_graphql::*;
use base64::{encode_config, URL_SAFE};
use sqlx::postgres::PgPool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Prefecture {
    pub id: i64,
    pub name: String,
}

#[Object]
impl Prefecture {
    async fn id(&self) -> ID {
        encode_config(format!("Prefecture:{}", self.id), URL_SAFE).into()
    }
    async fn name(&self) -> &str {
        &self.name
    }
}

#[tracing::instrument]
pub async fn get_prefectures(pool: &PgPool) -> anyhow::Result<Vec<Prefecture>> {
    let prefectures = sqlx::query_as::<_, Prefecture>(
        r#"
    SELECT * FROM prefectures
    "#,
    )
    .fetch_all(pool)
    .await;

    match prefectures {
        Ok(prefectures) => Ok(prefectures),
        Err(e) => {
            tracing::error!("prefectures fetch_all error");
            Err(e.into())
        }
    }
}
