use async_graphql::{Context, Object, Result};

use crate::{
    database::get_db_pool,
    graphql::{
        models::user::{register_user, User, Viewer},
        mutations::user_mutation::{RegisterUserInput, RegisterUserResult},
    },
};

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    #[allow(non_snake_case)]
    async fn viewer(&self, _ctx: &Context<'_>) -> Result<Viewer> {
        let user = User {
            id: 1,
            name: String::from("kosuda"),
            email: String::from("kosuda0428@gmail.com"),
            unverified_email: Some(String::from("kosuda0428@gmail.com")),
            avatar: String::from(
                "https://abs.twimg.com/sticky/default_profile_images/default_profile.png",
            ),
            role: crate::graphql::models::user::UserRole::General,
            introduction: None,
            email_verification_status:
                crate::graphql::models::user::EmailVerificationStatus::Pending,
        };

        let viewer = Viewer {
            account_user: Some(user),
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

        match input.register_user_validate().await {
            Some(errors) => return Ok(errors),
            None => (),
        }

        match input.check_already_exists_email(pool).await? {
            Some(error) => return Ok(error),
            None => (),
        }

        let result = register_user(pool, &input).await?;
        Ok(result)
    }
}
