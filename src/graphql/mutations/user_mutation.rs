use anyhow::Result;
use async_graphql::{Enum, InputObject, SimpleObject, Union};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use sqlx::PgPool;
use validator::{Validate, ValidationError};

use crate::graphql::models::user::{is_already_exists_email, Viewer};

static PASSWORD_FORMAT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?=.*?[a-zA-Z])(?=.*?\d)[a-zA-Z\d]{8,}$").unwrap());

//* RegisterUser */
#[derive(InputObject, Debug, Deserialize, Validate)]
pub struct RegisterUserInput {
    #[validate(length(max = 50, message = "名前は50文字以内で入力してください"))]
    pub name: String,
    #[validate(
        email(message = "メールアドレスを正しく入力してください"),
        length(max = 100, message = "メールアドレスは100文字以内で入力してください")
    )]
    pub email: String,
    #[graphql(secret)]
    #[validate(
        length(min = 8, message = "パスワードは8文字以上にしてください"),
        custom(
            function = "validate_password",
            message = "パスワードを正しく入力してください"
        )
    )]
    pub password: String,
}

impl RegisterUserInput {
    pub async fn register_user_validate(&self) -> Option<RegisterUserResult> {
        match self.validate() {
            Ok(_) => None,
            Err(e) => {
                let errors: Vec<RegisterUserInvalidInputError> = e
                    .field_errors()
                    .iter()
                    .map(|(key, val)| {
                        let error = &val[0];
                        RegisterUserInvalidInputError {
                            message: match &error.message {
                                Some(message) => message.to_string(),
                                None => String::from(""),
                            },
                            field: match *key {
                                "name" => RegisterUserInvalidInputField::Name,
                                "email" => RegisterUserInvalidInputField::Email,
                                "password" => RegisterUserInvalidInputField::Password,
                                &_ => todo!(),
                            },
                        }
                    })
                    .collect();
                Some(RegisterUserInvalidInputErrors { errors }.into())
            }
        }
    }
    pub async fn check_already_exists_email(
        &self,
        pool: &PgPool,
    ) -> Result<Option<RegisterUserResult>> {
        let is_exists = is_already_exists_email(&self.email, pool).await;
        match is_exists {
            Ok(is_exists) => {
                if is_exists {
                    tracing::error!("This email address already exists");
                    let error = RegisterUserAlreadyExistsEmailError {
                        message: String::from("このメールアドレスは既に存在します"),
                    };
                    return Ok(Some(error.into()));
                }
                Ok(None)
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                Err(e)
            }
        }
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    match PASSWORD_FORMAT.is_match(password) {
        Ok(bool) => {
            if !bool {
                return Err(ValidationError::new("Password format is incorrect"));
            }
            Ok(())
        }
        Err(_) => Err(ValidationError::new("regex is_match failed")),
    }
}

#[derive(Union)]
#[allow(clippy::enum_variant_names)]
pub enum RegisterUserResult {
    RegisterUserSuccess(RegisterUserSuccess),
    RegisterUserInvalidInputErrors(RegisterUserInvalidInputErrors),
    RegisterUserAlreadyExistsEmailError(RegisterUserAlreadyExistsEmailError),
}

#[derive(SimpleObject, Debug)]
pub struct RegisterUserSuccess {
    pub viewer: Viewer,
}

#[derive(SimpleObject, Debug)]
pub struct RegisterUserInvalidInputErrors {
    pub errors: Vec<RegisterUserInvalidInputError>,
}

#[derive(SimpleObject, Debug)]
pub struct RegisterUserAlreadyExistsEmailError {
    pub message: String,
}

#[derive(SimpleObject, Debug)]
pub struct RegisterUserInvalidInputError {
    pub message: String,
    pub field: RegisterUserInvalidInputField,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegisterUserInvalidInputField {
    Name,
    Email,
    Password,
}