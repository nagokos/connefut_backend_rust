use async_graphql::Interface;

use self::{
    recruitment_mutation::{
        CreateRecruitmentInvalidInputError, UpdateRecruitmentInvalidInputError,
    },
    tag_mutation::CreateTagAlreadyExistsNameError,
    user_mutation::{
        LoginUserAuthenticationError, LoginUserInvalidInputError, LoginUserNotFoundError,
        RegisterUserAlreadyExistsEmailError, RegisterUserInvalidInputError,
    },
};

pub mod recruitment_mutation;
pub mod tag_mutation;
pub mod user_mutation;

#[derive(Interface)]
#[graphql(field(name = "message", type = "String"))]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    RegisterUserAlreadyExistsEmailError(RegisterUserAlreadyExistsEmailError),
    RegisterUserInvalidInputError(RegisterUserInvalidInputError),
    LoginUserInvalidInputError(LoginUserInvalidInputError),
    LoginUserNotFoundError(LoginUserNotFoundError),
    LoginUserAuthenticationError(LoginUserAuthenticationError),
    CreateRecruitmentInvalidInputError(CreateRecruitmentInvalidInputError),
    UpdateRecruitmentInvalidInputError(UpdateRecruitmentInvalidInputError),
    CreateTagAlreadyExistsNameError(CreateTagAlreadyExistsNameError),
}
