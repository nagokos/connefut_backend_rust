use anyhow::Result;
use chrono::Local;
use sqlx::{PgPool, Postgres, Transaction};

use crate::graphql::auth::external::{AuthenticationProvider, UserInfo};

use super::user::User;

#[tracing::instrument]
// 送られてきたproviderとuidでユーザーを検索
pub async fn get_external_user_from_provider_and_uid(
    pool: &PgPool,
    uid: &str,
    provider: AuthenticationProvider,
) -> Result<Option<User>> {
    let sql = r#"
        SELECT *
        FROM users as u
        WHERE EXISTS (
            SELECT 1
            FROM authentications as a
            WHERE u.id = a.user_id
            AND a.provider = $1
            AND a.uid = $2
        )
    "#;

    let row = sqlx::query_as::<_, User>(sql)
        .bind(provider)
        .bind(uid)
        .fetch_optional(pool)
        .await;

    match row {
        Ok(data) => {
            tracing::info!("get external user from provider and uid successed!!");
            Ok(data)
        }
        Err(e) => {
            tracing::error!("get external user from provider and uid failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
// 認証情報の作成
pub async fn create_authentication(
    tx: &mut Transaction<'_, Postgres>,
    uid: &str,
    provider: AuthenticationProvider,
    user_id: i64,
) -> Result<()> {
    let sql = r#"
        INSERT INTO authentications
            (provider, uid, user_id, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5)
    "#;

    let now = Local::now();
    let row = sqlx::query(sql)
        .bind(provider)
        .bind(uid)
        .bind(user_id)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await;

    match row {
        Ok(_) => {
            tracing::info!("create authentication successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("create authentication failed: {:?}", e);
            Err(e.into())
        }
    }
}
