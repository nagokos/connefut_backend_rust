use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use async_graphql::{Enum, Object, ID};
use base64::{encode_config, URL_SAFE};
use chrono::{DateTime, Duration, Local};
use rand::Rng;
use sqlx::{postgres::PgRow, PgPool, Row};
use std::ops::Add;

use crate::graphql::{
    mail::sender::send_email_verification_code,
    mutations::user_mutation::{
        LoginUserInput, LoginUserNotFoundError, LoginUserResult, LoginUserSuccess,
        RegisterUserInput,
    },
};

#[derive(Clone, Copy, Enum, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "camelCase")]
pub enum UserRole {
    General,
    Admin,
}

#[derive(Clone, Copy, Enum, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "email_verification_status")]
#[sqlx(rename_all = "camelCase")]
pub enum EmailVerificationStatus {
    Pending,
    Verified,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub unverified_email: Option<String>,
    pub avatar: String,
    pub role: UserRole,
    pub introduction: Option<String>,
    pub email_verification_status: EmailVerificationStatus,
    pub email_verification_code: Option<String>,
    pub email_verification_code_expires_at: Option<DateTime<Local>>,
    pub password_digest: String,
}

#[Object]
impl User {
    pub async fn id(&self) -> ID {
        encode_config(format!("User:{}", self.id), URL_SAFE).into()
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn email(&self) -> &str {
        &self.email
    }
    async fn unverified_email(&self) -> Option<&str> {
        self.unverified_email.as_deref()
    }
    async fn avatar(&self) -> &str {
        &self.avatar
    }
    async fn role(&self) -> UserRole {
        self.role
    }
    async fn introduction(&self) -> Option<&str> {
        self.introduction.as_deref()
    }
    async fn email_verification_status(&self) -> EmailVerificationStatus {
        self.email_verification_status
    }
}

#[derive(Clone, Debug)]
pub struct Viewer {
    pub account_user: Option<User>,
}

#[Object]
impl Viewer {
    async fn account_user(&self) -> Option<User> {
        self.account_user.clone()
    }
}

#[tracing::instrument]
pub async fn get_user_from_id(pool: &PgPool, id: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT *
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id.parse::<i64>()?)
    .fetch_optional(pool)
    .await;

    match user {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument(skip(input))]
pub async fn create(pool: &PgPool, input: &RegisterUserInput) -> Result<User> {
    let sql = r#"
        INSERT INTO users
            (name, email, unverified_email, password_digest, email_verification_code,
                email_verification_code_expires_at, last_sign_in_at, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
    "#;

    let password_hash = generate_password_hash(input.password.as_bytes())?;
    let email_verification_code = generate_email_verification_code();
    let now = chrono::Local::now();
    let expires_at = now.add(Duration::days(1));

    let user = sqlx::query_as::<_, User>(sql)
        .bind(&input.name)
        .bind(&input.email)
        .bind(&input.email)
        .bind(password_hash)
        .bind(email_verification_code)
        .bind(expires_at)
        .bind(now)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await;

    match user {
        Ok(user) => {
            tracing::info!("Register user successed!!");
            match send_email_verification_code(&user).await {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("{:?}", e);
                    return Err(e);
                }
            }
            Ok(user)
        }
        Err(e) => {
            tracing::error!("Register user failed.");
            tracing::error!("{}", e.to_string());
            Err(e.into())
        }
    }
}

#[tracing::instrument(skip(email))]
pub async fn is_already_exists_email(email: &str, pool: &PgPool) -> Result<bool> {
    let is_exists = sqlx::query(
        r#"
        SELECT COUNT(DISTINCT id)
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
    .map(|row: PgRow| {
        let size: i64 = row.get("count");
        !matches!(size, 0)
    })
    .fetch_one(pool)
    .await?;

    Ok(is_exists)
}

fn generate_email_verification_code() -> String {
    let mut rng = rand::thread_rng();
    let mut code = String::from("");
    for _i in 0..=6 {
        code.push_str(&rng.gen_range(0..9).to_string());
    }
    code
}

fn generate_password_hash(password: &[u8]) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password, &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(e) => {
            tracing::error!("Password hash generation failed.");
            Err(anyhow!(e))
        }
    }
}
