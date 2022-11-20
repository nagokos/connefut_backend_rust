use anyhow::Result;
use chrono::Local;
use sqlx::{postgres::PgRow, PgPool, Row};

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
