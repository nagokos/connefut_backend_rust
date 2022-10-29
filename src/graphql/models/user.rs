use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use async_graphql::*;

use async_graphql::{Context, Enum, Object, ID};
use chrono::{DateTime, Duration, Local};
use rand::Rng;
use sqlx::{postgres::PgRow, PgPool, Row};
use std::ops::Add;

use crate::{
    database::get_db_pool,
    graphql::{
        self,
        auth::get_viewer,
        id_encode,
        loader::get_loaders,
        mail::sender::send_email_verification_code,
        mutations::user_mutation::RegisterUserInput,
        resolvers::{
            recruitment_resolver::{RecruitmentConnection, RecruitmentEdge},
            user_resolver::{FollowingConnection, UserEdge},
        },
        utils::pagination::{PageInfo, RecruitmentSearchParams, SearchParams},
        FieldGuard,
    },
};

use super::recruitment::{
    get_stocked_recruitments, get_user_recruitments, is_next_stocked_recruitment,
    is_next_user_recruitment, RecruitmentStatus,
};

/// 権限
#[derive(Clone, Copy, Enum, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    #[graphql]
    General,
    Admin,
}

/// メールアドレスの確認状態
#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "email_verification_status")]
#[sqlx(rename_all = "lowercase")]
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
/// ユーザー
impl User {
    pub async fn id(&self) -> ID {
        id_encode("User", self.id).into()
    }
    /// ユーザーの表示名
    async fn name(&self) -> &str {
        &self.name
    }
    /// ユーザーのメールアドレス
    #[graphql(guard = "FieldGuard::new(self.id)")]
    async fn email(&self) -> Option<async_graphql::Result<Option<&str>>> {
        Some(Ok(self.email.as_str().into()))
    }
    /// ユーザーが未確認のメールアドレス
    #[graphql(guard = "FieldGuard::new(self.id)")]
    async fn unverified_email(&self) -> Option<&str> {
        self.unverified_email.as_deref()
    }
    /// ユーザーのアバターURL
    async fn avatar(&self) -> &str {
        &self.avatar
    }
    /// ユーザーの権限
    #[graphql(guard = "FieldGuard::new(self.id)")]
    async fn role(&self) -> UserRole {
        self.role
    }
    /// ユーザーの自己紹介
    async fn introduction(&self) -> Option<&str> {
        self.introduction.as_deref()
    }
    /// ユーザーのメールアドレス確認状態
    #[graphql(guard = "FieldGuard::new(self.id)")]
    async fn email_verification_status(&self) -> EmailVerificationStatus {
        self.email_verification_status
    }
    // todo is_following_viewerの追加(このユーザーがviewerをフォローしているかを返す)
    // todo async fn is_following_viewer() -> Option<async_graphql::Result<bool>>

    // todo N+1に対応
    // このユーザーがViewerからフォローされているか
    async fn viewer_is_following(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let loaders = get_loaders(ctx).await;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Ok(false), // ログインしてない時は全てfalse
        };
        let viewer_is_following = loaders
            .following_loader
            .load_one([viewer.id, self.id])
            .await?;
        match viewer_is_following {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    async fn recruitments(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<ID>,
    ) -> async_graphql::Result<RecruitmentConnection> {
        let search_params = SearchParams::new(first, after)?;
        let pool = get_db_pool(ctx).await?;
        let recruitments = get_user_recruitments(pool, search_params, self.id).await?;

        let edges = if recruitments.is_empty() {
            None
        } else {
            let edges: Vec<RecruitmentEdge> = recruitments
                .iter()
                .map(|recruitment| RecruitmentEdge {
                    cursor: Default::default(),
                    node: recruitment.to_owned(),
                })
                .collect();
            Some(edges)
        };

        let page_info = match recruitments.last() {
            Some(recruitment) => {
                let has_next_page = is_next_user_recruitment(pool, recruitment.id, self.id).await?;
                let end_cursor =
                    encode_config(format!("Recruitment:{}", recruitment.id), base64::URL_SAFE);
                PageInfo {
                    has_next_page,
                    end_cursor: Some(end_cursor),
                    ..Default::default()
                }
            }
            None => Default::default(),
        };
        Ok(RecruitmentConnection { edges, page_info })
    }
    async fn following(
        &self,
        ctx: &Context<'_>,
        after: Option<ID>,
        first: Option<i32>,
    ) -> async_graphql::Result<FollowingConnection> {
        let pool = get_db_pool(ctx).await?;
        let params = SearchParams::new(first, after)?;

        let following = get_following(pool, self.id, params).await?;

        let edges: Vec<Option<UserEdge>> = following
            .iter()
            .map(|user| {
                UserEdge {
                    node: user.to_owned(),
                }
                .into()
            })
            .collect();

        let page_info = match following.last() {
            Some(user) => {
                let has_next_page = is_next_following_edge(pool, self.id, user.id).await?;
                let end_cursor = Some(id_encode("User", user.id));
                PageInfo {
                    has_next_page,
                    end_cursor,
                    ..Default::default()
                }
            }
            None => Default::default(),
        };

        Ok(FollowingConnection {
            edges: edges.into(),
            page_info,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Viewer {
    pub account_user: User,
}

#[Object]
impl Viewer {
    async fn account_user(&self) -> User {
        self.account_user.clone()
    }
    async fn recruitments(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<ID>,
    ) -> async_graphql::Result<RecruitmentConnection> {
        let pool = get_db_pool(ctx).await?;
        let search_params = SearchParams::new(first, after)?;
        let recruitments =
            get_viewer_recruitments(pool, search_params, self.account_user.id).await?;

        let edges = if recruitments.is_empty() {
            None
        } else {
            let edges: Vec<RecruitmentEdge> = recruitments
                .iter()
                .map(|recruitment| RecruitmentEdge {
                    cursor: String::default(),
                    node: recruitment.to_owned(),
                })
                .collect();
            Some(edges)
        };

        let page_info = match recruitments.last() {
            Some(recruitment) => {
                let has_next_page =
                    is_next_viewer_recruitment(pool, recruitment.id, self.account_user.id).await?;
                let end_cursor = id_encode("Recruitment", recruitment.id);
                PageInfo {
                    has_next_page,
                    end_cursor: Some(end_cursor),
                    ..Default::default()
                }
            }
            None => Default::default(),
        };
        Ok(RecruitmentConnection { edges, page_info })
    }
    async fn stocked_recruitments(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<ID>,
    ) -> Result<RecruitmentConnection> {
        let pool = get_db_pool(ctx).await?;
        let search_params = SearchParams::new(first, after)?;

        let recruitments =
            get_stocked_recruitments(pool, self.account_user.id, search_params).await?;

        let edges = if recruitments.is_empty() {
            None
        } else {
            let edges: Vec<RecruitmentEdge> = recruitments
                .iter()
                .map(|recruitment| RecruitmentEdge {
                    cursor: String::default(),
                    node: recruitment.to_owned(),
                })
                .collect();
            Some(edges)
        };

        let page_info = match recruitments.last() {
            Some(recruitment) => {
                let end_cursor = id_encode("Recruitment", recruitment.id);
                let has_next_page =
                    is_next_stocked_recruitment(pool, recruitment.id, self.account_user.id).await?;
                PageInfo {
                    end_cursor: Some(end_cursor),
                    has_next_page,
                    ..Default::default()
                }
            }
            None => Default::default(),
        };

        Ok(RecruitmentConnection { edges, page_info })
    }
}

#[tracing::instrument]
pub async fn get_user_from_id(pool: &PgPool, id: i64) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT *
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
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
pub async fn get_user_from_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT *
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(email)
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

pub fn authentication(password: &[u8], password_hash: &str) -> Result<bool> {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("{:?}", e);
            return Err(anyhow!(e));
        }
    };
    let is_auth = Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok();
    Ok(is_auth)
}

// ? bool返す時isつけなくちゃいけないか他のネーミング探す
#[tracing::instrument(skip(email))]
pub async fn is_already_exists_email(email: &str, pool: &PgPool) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT *
            FROM users
            WHERE email = $1
        )
    "#;
    let row = sqlx::query(sql)
        .bind(email)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_exists) => {
            tracing::info!("is already exists email successed!!");
            Ok(is_exists)
        }
        Err(e) => {
            tracing::error!("is already exists email failed: {:?}", e);
            Err(e.into())
        }
    }
}

fn generate_email_verification_code() -> String {
    let mut rng = rand::thread_rng();
    let mut code = String::from("");
    for _i in 0..6 {
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

#[tracing::instrument]
pub async fn is_already_following(
    pool: &PgPool,
    follower_id: i64,
    followed_id: i64,
) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT id
            FROM relationships
            WHERE follower_id = $1
            AND followed_id = $2
        )
    "#;

    let row = sqlx::query(sql)
        .bind(follower_id)
        .bind(followed_id)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_following) => {
            tracing::info!("is already following successed!!");
            Ok(is_following)
        }
        Err(e) => {
            tracing::error!("is already following failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn get_following(
    pool: &PgPool,
    follower_id: i64,
    params: SearchParams,
) -> Result<Vec<User>> {
    let sql = r#"
        SELECT u.*
        FROM users as u
        INNER JOIN relationships as r
            ON u.id = r.followed_id
        WHERE r.follower_id = $1
        AND ($2 OR r.id < (SELECT id
                           FROM relationships
                           WHERE follower_id = $3
                           AND followed_id = $4))
        ORDER BY r.id DESC
        LIMIT $5
    "#;

    let rows = sqlx::query_as::<_, User>(sql)
        .bind(follower_id)
        .bind(!params.use_after)
        .bind(follower_id)
        .bind(params.after)
        .bind(params.num_rows)
        .fetch_all(pool)
        .await;

    match rows {
        Ok(following) => {
            tracing::info!("get following successed!!");
            Ok(following)
        }
        Err(e) => {
            tracing::error!("get following failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn is_next_following_edge(
    pool: &PgPool,
    follower_id: i64,
    followed_id: i64,
) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT id
            FROM relationships
            WHERE id < (SELECT id 
                        FROM relationships 
                        WHERE follower_id = $1 
                        AND followed_id = $2)
            ORDER BY id DESC
            LIMIT 1
        )
    "#;

    let row = sqlx::query(sql)
        .bind(follower_id)
        .bind(followed_id)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_next) => {
            tracing::info!("is next following edge successed!!");
            Ok(is_next)
        }
        Err(e) => {
            tracing::error!("is next following edge failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn follow(pool: &PgPool, follower_id: i64, followed_id: i64) -> Result<()> {
    let sql = r#"
        INSERT INTO relationships
            (follower_id, followed_id, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4)
    "#;

    let now = Local::now();
    let row = sqlx::query(sql)
        .bind(follower_id)
        .bind(followed_id)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await;

    match row {
        Ok(_) => {
            tracing::info!("follow user successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("follow user failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn unfollow(pool: &PgPool, follower_id: i64, followed_id: i64) -> Result<()> {
    let sql = r#"
        DELETE FROM relationships 
        WHERE follower_id = $1
        AND followed_id = $2
    "#;

    let row = sqlx::query(sql)
        .bind(follower_id)
        .bind(followed_id)
        .execute(pool)
        .await;

    match row {
        Ok(_) => {
            tracing::info!("unfollow user successed!!");
            Ok(())
        }
        Err(e) => {
            tracing::error!("unfollow user failed: {:?}", e);
            Err(e.into())
        }
    }
}
