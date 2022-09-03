use anyhow::{anyhow, Result};
use async_graphql::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

use crate::config::Config;

pub async fn pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(config.database.url.as_str())
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
