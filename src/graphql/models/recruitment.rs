use anyhow::Result;
use async_graphql::{Context, Enum, Object, ID};
use base64::{encode_config, URL_SAFE};
use chrono::{DateTime, Local};
use sqlx::PgPool;

use crate::{
    database::get_db_pool,
    graphql::{id_decode, mutations::recruitment_mutation::RecruitmentInput},
};

use super::{
    tag::{
        add_recruitment_tags, add_recruitment_tags_tx, get_recruitment_tags,
        remove_recruitment_tags_tx, Tag,
    },
    user::{get_user_from_id, User},
};

#[derive(Enum, Clone, Copy, Eq, PartialEq, Debug, sqlx::Type)]
#[sqlx(type_name = "recruitment_category")]
#[sqlx(rename_all = "lowercase")]
pub enum Category {
    Opponent,
    Personal,
    Member,
    Join,
    Other,
}

#[derive(Enum, Clone, Copy, Eq, PartialEq, Debug, sqlx::Type)]
#[sqlx(type_name = "recruitment_status")]
#[sqlx(rename_all = "lowercase")]
pub enum Status {
    Draft,
    Published,
    Closed,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Recruitment {
    pub id: i64,
    pub title: String,
    pub category: Category,
    pub venue: Option<String>,
    pub venue_lat: Option<f64>,
    pub venue_lng: Option<f64>,
    pub start_at: Option<DateTime<Local>>,
    pub closing_at: Option<DateTime<Local>>,
    pub detail: Option<String>,
    pub sport_id: i64,
    pub prefecture_id: i64,
    pub status: Status,
    pub user_id: i64,
    pub published_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
}

#[Object]
impl Recruitment {
    pub async fn id(&self) -> ID {
        encode_config(format!("Recruitment:{}", self.id), URL_SAFE).into()
    }
    pub async fn title(&self) -> &str {
        &self.title
    }
    pub async fn category(&self) -> Category {
        self.category
    }
    pub async fn venue(&self) -> Option<&str> {
        self.venue.as_deref()
    }
    pub async fn venue_lat(&self) -> Option<f64> {
        self.venue_lat
    }
    pub async fn venue_lng(&self) -> Option<f64> {
        self.venue_lng
    }
    pub async fn start_at(&self) -> Option<DateTime<Local>> {
        self.start_at
    }
    pub async fn closing_at(&self) -> Option<DateTime<Local>> {
        self.closing_at
    }
    pub async fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
    pub async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let pool = get_db_pool(ctx).await?;
        let user = get_user_from_id(pool, self.user_id).await?;
        match user {
            Some(u) => Ok(u),
            None => Err(async_graphql::Error::new(String::from("User not found"))),
        }
    }
    pub async fn created_at(&self) -> DateTime<Local> {
        self.created_at
    }
    pub async fn status(&self) -> Status {
        self.status
    }
    pub async fn tags(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Tag>> {
        let pool = get_db_pool(ctx).await?;
        let tags = get_recruitment_tags(pool, self.id).await?;
        Ok(tags)
    }
}

#[tracing::instrument]
pub async fn create(pool: &PgPool, input: RecruitmentInput, user_id: i64) -> Result<Recruitment> {
    let sql = r#"
      INSERT INTO recruitments
        (title, category, venue, venue_lat, venue_lng, start_at, closing_at, 
            detail, sport_id, prefecture_id, status, user_id, published_at, created_at, updated_at)
      VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
      RETURNING *
    "#;

    let now = Local::now();
    let published_at = match input.status {
        Status::Published => Some(now),
        _ => None,
    };

    let row = sqlx::query_as::<_, Recruitment>(sql)
        .bind(input.title)
        .bind(input.category)
        .bind(input.venue)
        .bind(input.venue_lat)
        .bind(input.venue_lng)
        .bind(input.start_at)
        .bind(input.closing_at)
        .bind(input.detail)
        .bind(id_decode(&input.sport_id)?)
        .bind(id_decode(&input.prefecture_id)?)
        .bind(input.status)
        .bind(user_id)
        .bind(published_at)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await;

    match row {
        Ok(recruitment) => {
            if !input.tag_ids.is_empty() {
                add_recruitment_tags(pool, input.tag_ids, recruitment.id).await?;
            }
            Ok(recruitment)
        }
        Err(e) => {
            tracing::error!("{}", e.to_string());
            Err(e.into())
        }
    }
}
