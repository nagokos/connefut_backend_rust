use chrono::prelude::*;

use serde::Deserialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::env;

use connefut_api::{config::get_config, database::pool};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct ResultItem {
    prefName: String,
}

#[derive(Deserialize, Debug)]
struct Result {
    result: Vec<ResultItem>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let config = get_config();
    let pool = pool(config).await?;
    insert_initial_prefectures_data(&pool).await?;
    insert_initial_sports_data(&pool).await?;
    insert_initial_tags_data(&pool).await?;

    tracing::info!("insert all data!!");
    Ok(())
}

#[tracing::instrument]
async fn insert_initial_prefectures_data(pool: &PgPool) -> anyhow::Result<()> {
    let prefectures = get_prefectures().await?;

    let mut builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO prefectures(name, created_at, updated_at)");
    builder.push_values(prefectures.result, |mut b, prefecture| {
        let now = Local::now();
        b.push_bind(prefecture.prefName)
            .push_bind(now)
            .push_bind(now);
    });
    builder.push("RETURNING id, name");

    builder.build().execute(pool).await?;

    tracing::info!("init prefectures data!!");
    Ok(())
}

#[tracing::instrument]
async fn get_prefectures() -> anyhow::Result<Result> {
    let client = reqwest::Client::builder().https_only(true).build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application-json".parse()?);

    let api_key = dotenvy::var("API_KEY")?;
    headers.insert("X-API-KEY", api_key.as_str().parse()?);

    let resp = client
        .get("https://opendata.resas-portal.go.jp/api/v1/prefectures")
        .headers(headers)
        .send()
        .await?;
    let body = resp.text().await?;
    let json: Result = serde_json::from_str(&body)?;
    Ok(json)
}

#[tracing::instrument]
async fn insert_initial_sports_data(pool: &PgPool) -> anyhow::Result<()> {
    let sports: [&str; 3] = ["サッカー", "フットサル", "ソサイチ"];

    let mut builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO sports(name, created_at, updated_at)");
    builder.push_values(sports, |mut b, sport| {
        let now = Local::now();
        b.push_bind(sport).push_bind(now).push_bind(now);
    });

    builder.build().execute(pool).await?;

    tracing::info!("init sports data!!");
    Ok(())
}

#[tracing::instrument]
async fn insert_initial_tags_data(pool: &PgPool) -> anyhow::Result<()> {
    let tags: [&str; 10] = [
        "エンジョイ",
        "男女mix",
        "シニア",
        "ガチ",
        "誰でもok",
        "経験者",
        "初心者歓迎",
        "競技志向",
        "急募",
        "人工芝",
    ];

    let mut builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO tags(name, created_at, updated_at)");
    builder.push_values(tags, |mut b, tag| {
        let now = Local::now();
        b.push_bind(tag).push_bind(now).push_bind(now);
    });

    builder.build().execute(pool).await?;

    tracing::info!("init sports data!!");
    Ok(())
}
