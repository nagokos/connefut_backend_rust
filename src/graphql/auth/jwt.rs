use std::ops::Add;

use anyhow::Result;
use async_graphql::Context;
use chrono::{Duration, Local};
use cookie::{time, Cookie, SameSite};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::graphql::models::user::{get_user_from_id, User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: i64,
    pub iat: i64,
    pub sub: String,
}

impl Default for Claims {
    fn default() -> Self {
        Self {
            exp: Local::now().add(Duration::days(1)).timestamp(),
            iat: Local::now().timestamp(),
            sub: String::default(),
        }
    }
}

// keyは別途用意してawsのsecretmanager的なのに保存(開発環境、本番で分ける)
pub fn token_encode(claims: Claims) -> Result<String> {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )?;
    Ok(token)
}

pub fn token_decode(token: String) -> Result<TokenData<Claims>> {
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    ) {
        Ok(c) => Ok(c),
        Err(e) => {
            tracing::error!("{:?}", e);
            Err(e.into())
        }
    }
}

// todo エラー返すようにする
pub async fn get_user_from_token(pool: &PgPool, token: String) -> Option<User> {
    match token_decode(token) {
        Ok(token_data) => {
            match get_user_from_id(pool, token_data.claims.sub.parse::<i64>().ok()?).await {
                Ok(user) => user,
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub fn set_jwt_cookie(jwt: String, ctx: &Context<'_>) {
    let cookie = Cookie::build("token", jwt)
        .path("/")
        .http_only(true)
        .max_age(time::Duration::DAY)
        .same_site(SameSite::Lax)
        .finish();
    ctx.append_http_header("Set-Cookie", cookie.to_string());
}
