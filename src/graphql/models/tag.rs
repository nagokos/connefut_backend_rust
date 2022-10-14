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
pub async fn add_recruitment_tags_tx(
    tx: &mut Transaction<'_, Postgres>,
    tag_ids: Vec<i64>,
    recruitment_id: i64,
) -> Result<()> {
    if tag_ids.is_empty() {
        return Ok(());
    };

    let sql = "INSERT INTO recruitment_tags(recruitment_id, tag_id, created_at, updated_at) ";

    let mut query_builder = QueryBuilder::<Postgres>::new(sql);
    query_builder.push_values(tag_ids, |mut b, tag_id| {
        let now = Local::now();
        b.push_bind(recruitment_id)
            .push_bind(tag_id)
            .push_bind(now)
            .push_bind(now);
    });
    let query = query_builder.build();

    let result = query.execute(tx).await;

    match result {
        Ok(_) => {
            tracing::info!("add recruitment tags successed!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            tracing::error!("add recruitment tags failed...");
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn remove_recruitment_tags_tx(
    tx: &mut Transaction<'_, Postgres>,
    tag_ids: Vec<i64>,
    recruitment_id: i64,
) -> Result<()> {
    if tag_ids.is_empty() {
        return Ok(());
    }

    let sql = "DELETE FROM recruitment_tags WHERE (tag_id, recruitment_id) IN";

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(sql.to_string());
    query_builder.push_tuples(tag_ids, |mut b, id| {
        b.push_bind(id).push_bind(recruitment_id);
    });

    let query = query_builder.build();

    match query.execute(tx).await {
        Ok(_) => {
            tracing::info!("remove recruitment tags successed!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("remove recruitment tags failed...");
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn create(pool: &PgPool, name: &str) -> Result<Tag> {
    let sql = r#"
        INSERT INTO tags
            (name, created_at, updated_at)
        VALUES
            ($1, $2, $3)
        RETURNING *
    "#;

    let now = Local::now();
    let row = sqlx::query_as::<_, Tag>(sql)
        .bind(name)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await;

    match row {
        Ok(tag) => {
            tracing::info!("create tag successed!");
            Ok(tag)
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn get_tags(pool: &PgPool) -> Result<Vec<Tag>> {
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

#[tracing::instrument]
pub async fn get_recruitment_tags(pool: &PgPool, recruitment_id: i64) -> Result<Vec<Tag>> {
    let sql = r#"
      SELECT t.*
      FROM tags as t
      INNER JOIN recruitment_tags as r_t
      ON t.id = r_t.tag_id
      WHERE r_t.recruitment_id = $1
    "#;

    let rows = sqlx::query_as::<_, Tag>(sql)
        .bind(recruitment_id)
        .fetch_all(pool)
        .await;

    match rows {
        Ok(tags) => {
            tracing::info!("get recruitment tags successed!");
            Ok(tags)
        }
        Err(e) => {
            tracing::error!("get recruitment tags failed...");
            tracing::error!("{}", e);
            Err(e.into())
        }
    }
}

// todo tracing書く
#[tracing::instrument]
pub async fn is_already_exists_tag_name(pool: &PgPool, name: &str) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT *
            FROM tags
            WHERE name = $1
        )
    "#;

    let row = sqlx::query(sql)
        .bind(name)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_exists) => {
            tracing::info!("is already exists tag name successed!!");
            Ok(is_exists)
        }
        Err(e) => {
            tracing::error!("is already exists tag name failed: {:?}", e);
            Err(e.into())
        }
    }
}
