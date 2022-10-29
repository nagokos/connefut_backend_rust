use async_graphql::{Context, Object, Result, SimpleObject, ID};

use crate::{
    database::get_db_pool,
    graphql::{
        auth::{
            get_viewer,
            jwt::{self, Claims},
        },
        id_decode, id_encode,
        mail::sender::send_email_verification_code,
        models::user::{
            self, authentication, follow, get_user_from_email, get_user_from_id, unfollow, User,
        },
        mutations::user_mutation::{
            FollowUserInput, FollowUserResult, FollowUserSuccess, LoginUserAuthenticationError,
            LoginUserInput, LoginUserNotFoundError, LoginUserResult, LoginUserSuccess,
            RegisterUserInput, RegisterUserResult, RegisterUserSuccess, UnfollowUserInput,
            UnfollowUserResult,
        },
        utils::pagination::PageInfo,
    },
};

#[derive(SimpleObject)]
pub struct FollowingConnection {
    pub edges: Option<Vec<Option<UserEdge>>>,
    pub page_info: PageInfo,
}

pub struct UserEdge {
    pub node: User,
}

#[Object]
impl UserEdge {
    async fn cursor(&self) -> ID {
        id_encode("User", self.node.id).into()
    }
    async fn node(&self) -> Option<User> {
        self.node.clone().into()
    }
}

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    /// 現在認証されているユーザーを取得する
    async fn viewer(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let user = get_viewer(ctx).await;
        Ok(user.to_owned())
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    /// ユーザーを新規登録する
    async fn register_user(
        &self,
        ctx: &Context<'_>,
        input: RegisterUserInput,
    ) -> Result<RegisterUserResult> {
        let pool = get_db_pool(ctx).await?;

        if let Some(errors) = input.register_user_validate() {
            return Ok(errors.into());
        }

        if let Some(error) = input.check_already_exists_email(pool).await? {
            return Ok(error.into());
        }

        let user = user::create(pool, &input).await?;

        send_email_verification_code(&user).await.map_err(|e| {
            tracing::error!("send email verification code failed: {:?}", e);
            e
        })?;

        let claims = Claims {
            sub: user.id.to_string(),
            ..Default::default()
        };
        match jwt::token_encode(claims) {
            Ok(token) => {
                jwt::set_jwt_cookie(token, ctx);
                let viewer = Viewer { account_user: user };
                Ok(RegisterUserSuccess { viewer }.into())
            }
            Err(e) => Err(e.into()),
        }
    }
    /// ユーザーを認証する
    async fn login_user(
        &self,
        ctx: &Context<'_>,
        input: LoginUserInput,
    ) -> Result<LoginUserResult> {
        let pool = get_db_pool(ctx).await?;

        if let Some(errors) = input.login_user_validate() {
            return Ok(errors.into());
        }

        let user = match get_user_from_email(pool, &input.email).await? {
            Some(user) => user,
            None => {
                let not_found = LoginUserNotFoundError {
                    message: String::from("メールアドレス又はパスワードが正しくありません"),
                };
                tracing::error!("user not found");
                return Ok(not_found.into());
            }
        };

        match authentication(input.password.as_bytes(), &user.password_digest) {
            Ok(is_auth) => {
                if !is_auth {
                    let auth_error = LoginUserAuthenticationError {
                        message: String::from("メールアドレス、またはパスワードが正しくありません"),
                    };
                    tracing::error!("Failed to authenticate user");
                    return Ok(auth_error.into());
                }

                let claims = Claims {
                    sub: user.id.to_string(),
                    ..Default::default()
                };
                match jwt::token_encode(claims) {
                    Ok(token) => {
                        jwt::set_jwt_cookie(token, ctx);
                        let viewer = Viewer { account_user: user };
                        tracing::info!("User authenticated.");
                        Ok(LoginUserSuccess { viewer }.into())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }
    /// ユーザーをフォローする
    async fn follow_user(
        &self,
        ctx: &Context<'_>,
        input: FollowUserInput,
    ) -> Result<FollowUserResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        if let Some(e) = input.check_has_already_following(pool, viewer.id).await? {
            return Ok(e.into());
        }

        let user_id = id_decode(&input.user_id)?;
        follow(pool, viewer.id, user_id).await?;
        let user = match get_user_from_id(pool, user_id).await? {
            Some(user) => user,
            None => {
                tracing::error!("user not found...");
                return Err(async_graphql::Error::new("user not found..."));
            }
        };

        let success = FollowUserSuccess { user };
        Ok(success.into())
    }
    /// ユーザーのフォローを外す
    async fn unfollow_user(
        &self,
        ctx: &Context<'_>,
        input: UnfollowUserInput,
    ) -> Result<UnfollowUserResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let user_id = id_decode(&input.user_id)?;
        unfollow(pool, viewer.id, user_id).await?;
        let user = match get_user_from_id(pool, user_id).await? {
            Some(user) => user,
            None => {
                tracing::error!("user not found");
                return Err(async_graphql::Error::new("user not found"));
            }
        };

        Ok(UnfollowUserResult { user })
    }
}
