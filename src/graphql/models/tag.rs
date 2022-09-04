use async_graphql::{Object, ID};
use base64::{encode_config, URL_SAFE};
use sqlx::PgPool;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[Object]
impl Tag {
    pub async fn id(&self) -> ID {
        encode_config(format!("Tag:{}", self.id), URL_SAFE).into()
    }
    async fn name(&self) -> &str {
        &self.name
    }
}

#[tracing::instrument]
pub async fn get_tags(pool: &PgPool) -> anyhow::Result<Vec<Tag>> {
    let tags = sqlx::query_as::<_, Tag>(
        r#"
        SELECT * FROM tags
        "#,
    )
    .fetch_all(pool)
    .await;

    match tags {
        Ok(tags) => Ok(tags),
        Err(e) => {
            tracing::error!("get_tags fetch_all error");
            Err(e.into())
        }
    }
}
