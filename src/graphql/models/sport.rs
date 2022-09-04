use async_graphql::*;
use base64::{encode_config, URL_SAFE};
use sqlx::PgPool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Sport {
    pub id: i64,
    pub name: String,
}

#[Object]
impl Sport {
    pub async fn id(&self) -> ID {
        encode_config(format!("Sport:{}", self.id), URL_SAFE).into()
    }
    async fn name(&self) -> &str {
        &self.name
    }
}

#[tracing::instrument]
pub async fn get_sports(pool: &PgPool) -> anyhow::Result<Vec<Sport>> {
    let sports = sqlx::query_as::<_, Sport>(
        r#"
        SELECT * FROM sports
        "#,
    )
    .fetch_all(pool)
    .await;

    match sports {
        Ok(sports) => Ok(sports),
        Err(e) => {
            tracing::error!("sports fetch_all error");
            Err(e.into())
        }
    }
}
