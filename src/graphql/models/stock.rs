use anyhow::Result;
use async_graphql::{Object, ID};
use chrono::Local;
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::graphql::id_encode;

#[derive(Clone, Copy)]
pub struct Stock {
    pub recruitment_id: i64,
    pub viewer_has_stocked: bool,
}

#[Object]
impl Stock {
    pub async fn id(&self) -> ID {
        id_encode("Stock", self.recruitment_id).into()
    }
    pub async fn viewer_has_stocked(&self) -> bool {
        self.viewer_has_stocked
    }
}

pub async fn is_already_stocked(pool: &PgPool, user_id: i64, recruitment_id: i64) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT id
            FROM stocks
            WHERE user_id = $1
            AND recruitment_id = $2
        )
    "#;

    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(recruitment_id)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_stocked) => {
            tracing::info!("is already stocked successed!!");
            Ok(is_stocked)
        }
        Err(e) => {
            tracing::error!("is already stocked failed: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn add_stock(pool: &PgPool, user_id: i64, recruitment_id: i64) -> Result<()> {
    let sql = r#"
        INSERT INTO stocks
            (user_id, recruitment_id, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4)
    "#;

    let now = Local::now();
    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(recruitment_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await;

    match row {
        Ok(_) => {
            tracing::info!("add stock successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("add stock failed: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn remove_stock(pool: &PgPool, user_id: i64, recruitment_id: i64) -> Result<()> {
    let sql = r#"
        DELETE FROM stocks 
        WHERE user_id = $1 
        AND recruitment_id = $2
    "#;

    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(recruitment_id)
        .execute(pool)
        .await;

    match row {
        Ok(_) => {
            tracing::info!("remove stock successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("remove stock failed: {:?}", e);
            Err(e.into())
        }
    }
}
