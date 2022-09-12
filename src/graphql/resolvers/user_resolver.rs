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

        // todo if letでいいと思う
        match input.register_user_validate().await {
            Some(errors) => return Ok(errors.into()),
            None => (),
        }
        // todo if letでいいと思う
        match input.check_already_exists_email(pool).await? {
            Some(error) => return Ok(error.into()),
            None => (),
        }

        let user = user::create(pool, &input).await?;

        let claims = Claims {
            sub: user.id.to_string(),
            ..Default::default()
        };
        match jwt::decode(claims) {
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
}
