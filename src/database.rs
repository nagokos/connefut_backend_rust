use anyhow::anyhow;
use async_graphql::Context;
use once_cell::sync::Lazy;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc};
struct Config {
    postgres_host: String,
    postgres_port: String,
    postgres_user: String,
    postgres_password: String,
    postgres_database: String,
    postgres_sslmode: String,
}

impl Config {
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_database,
            self.postgres_sslmode,
        )
    }
}

static CONFIG: Lazy<Config> = Lazy::new(|| Config {
    postgres_host: env::var("POSTGRES_HOST").unwrap(),
    postgres_user: env::var("POSTGRES_USER").unwrap(),
    postgres_database: env::var("POSTGRES_DB").unwrap(),
    postgres_port: env::var("POSTGRES_PORT").unwrap(),
    postgres_password: env::var("POSTGRES_PASSWORD").unwrap(),
    postgres_sslmode: env::var("PGSSLMODE").unwrap(),
});

pub async fn pool() -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&CONFIG.database_url())
        .await;

    match pool {
        Ok(pool) => anyhow::Ok(pool),
        Err(e) => {
            tracing::error!("db connection failed");
            Err(e.into())
        }
    }
}

pub async fn get_db_pool<'ctx>(ctx: &Context<'ctx>) -> anyhow::Result<&'ctx Arc<PgPool>> {
    match ctx.data::<Arc<PgPool>>() {
        Ok(pool) => Ok(pool),
        Err(e) => {
            tracing::error!("get db pool error");
            Err(anyhow!(e.message))
        }
    }
}
