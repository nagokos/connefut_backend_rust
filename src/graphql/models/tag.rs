use anyhow::Result;
use async_graphql::{Object, ID};
use base64::{encode_config, URL_SAFE};
use chrono::Local;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row, Transaction};

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

// todo poolとtransactionを両方受け取れるようにする

#[tracing::instrument]
pub async fn add_recruitment_tags(
    pool: &PgPool,
    tag_ids: Vec<i64>,
    recruitment_id: i64,
) -> Result<()> {
    if tag_ids.is_empty() {
        return Ok(());
    }

    let sql = "INSERT INTO recruitment_tags(tag_id, recruitment_id, created_at, updated_at) ";
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(sql.to_string());

    query_builder.push_values(tag_ids, |mut b, id| {
        let now = Local::now();
        b.push_bind(id)
            .push_bind(recruitment_id)
            .push_bind(now)
            .push_bind(now);
    });

    let query = query_builder.build();

    match query.execute(pool).await {
        Ok(_) => {
            tracing::info!("add recruitment tags successed!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("add recruitment tags failed...");
            tracing::error!("{}", e);
            Err(e.into())
        }
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
