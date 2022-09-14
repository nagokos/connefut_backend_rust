use async_graphql::{Context, Object, Result};

use crate::{
    database::get_db_pool,
    graphql::{
        auth::{
            get_viewer,
            jwt::{self, Claims},
        },
        models::user::{self, authentication, get_user_from_email, Viewer},
        mutations::user_mutation::{
            LoginUserAuthenticationError, LoginUserInput, LoginUserNotFoundError, LoginUserResult,
            LoginUserSuccess, RegisterUserInput, RegisterUserResult, RegisterUserSuccess,
        },
    },
};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    #[allow(non_snake_case)]
    async fn viewer(&self, ctx: &Context<'_>) -> Result<Viewer> {
        let user = get_viewer(ctx).await;
        let viewer = Viewer {
            account_user: { user.to_owned() },
        };
        Ok(viewer)
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    #[allow(non_snake_case)]
    async fn registerUser(
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

        let claims = Claims {
            sub: user.id.to_string(),
            ..Default::default()
        };
        match jwt::token_encode(claims) {
            Ok(token) => {
                jwt::set_jwt_cookie(token, ctx);
                let viewer = Viewer {
                    account_user: user.into(),
                };
                Ok(RegisterUserSuccess { viewer }.into())
            }
            Err(e) => Err(e.into()),
        }
    }
    #[allow(non_snake_case)]
    async fn loginUser(&self, ctx: &Context<'_>, input: LoginUserInput) -> Result<LoginUserResult> {
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
                        let viewer = Viewer {
                            account_user: user.into(),
                        };
                        tracing::info!("User authenticated.");
                        Ok(LoginUserSuccess { viewer }.into())
                    }
                    Err(e) => Err(e.into()),
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
