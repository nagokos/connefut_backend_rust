use anyhow::Result;
use async_graphql::{Enum, InputObject, SimpleObject, Union, ID};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use sqlx::PgPool;
use validator::{Validate, ValidationError};

use crate::graphql::{
    id_decode,
    models::user::{is_already_exists_email, is_already_following, User, Viewer},
};

static PASSWORD_FORMAT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?=.*?[a-zA-Z])(?=.*?\d)[a-zA-Z\d]{8,}$").unwrap());

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

//* RegisterUser */
#[derive(InputObject, Debug, Validate)]
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
    pub fn register_user_validate(&self) -> Option<RegisterUserInvalidInputErrors> {
        match self.validate() {
            Ok(_) => None,
            Err(e) => {
                let errors: Vec<RegisterUserInvalidInputError> = e
                    .field_errors()
                    .iter()
                    .map(|(key, val)| {
                        let error = &val[0];
                        RegisterUserInvalidInputError {
                            message: match error.message {
                                Some(ref message) => message.to_string(),
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
                Some(RegisterUserInvalidInputErrors { errors })
            }
        }
    }
    pub async fn check_already_exists_email(
        &self,
        pool: &PgPool,
    ) -> Result<Option<RegisterUserAlreadyExistsEmailError>> {
        if is_already_exists_email(&self.email, pool).await? {
            tracing::error!("This email address already exists");
            let error = RegisterUserAlreadyExistsEmailError {
                message: String::from("このメールアドレスは既に存在します"),
            };
            Ok(Some(error))
        } else {
            Ok(None)
        }
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

//* LoginUser */
#[derive(InputObject, Debug, Validate)]
pub struct LoginUserInput {
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

impl LoginUserInput {
    pub fn login_user_validate(&self) -> Option<LoginUserInvalidInputErrors> {
        match self.validate() {
            Ok(_) => None,
            Err(e) => {
                let errors: Vec<LoginUserInvalidInputError> = e
                    .field_errors()
                    .iter()
                    .map(|(key, val)| {
                        let error = &val[0]; // fieldに対して複数エラーがあっても最初の一つだけ
                        LoginUserInvalidInputError {
                            message: match error.message {
                                Some(ref message) => message.to_string(),
                                None => String::from(""),
                            },
                            field: match *key {
                                "email" => LoginUserInvalidInputField::Email,
                                "password" => LoginUserInvalidInputField::Password,
                                &_ => todo!(),
                            },
                        }
                    })
                    .collect();
                Some(LoginUserInvalidInputErrors { errors })
            }
        }
    }
}

#[derive(Union)]
#[allow(clippy::enum_variant_names)]
pub enum LoginUserResult {
    LoginUserSuccess(LoginUserSuccess),
    LoginUserInvalidInputErrors(LoginUserInvalidInputErrors),
    LoginUserNotFoundError(LoginUserNotFoundError),
    LoginUserAuthenticationError(LoginUserAuthenticationError),
}

#[derive(SimpleObject, Debug)]
pub struct LoginUserSuccess {
    pub viewer: Viewer,
}

#[derive(SimpleObject, Debug)]
pub struct LoginUserNotFoundError {
    pub message: String,
}
#[derive(SimpleObject, Debug)]
pub struct LoginUserAuthenticationError {
    pub message: String,
}

#[derive(SimpleObject, Debug)]
pub struct LoginUserInvalidInputErrors {
    pub errors: Vec<LoginUserInvalidInputError>,
}

#[derive(SimpleObject, Debug)]
pub struct LoginUserInvalidInputError {
    pub message: String,
    pub field: LoginUserInvalidInputField,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoginUserInvalidInputField {
    Name,
    Email,
    Password,
}
