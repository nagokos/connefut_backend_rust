use async_graphql::Interface;

use self::user_mutation::{RegisterUserAlreadyExistsEmailError, RegisterUserInvalidInputError};

pub mod user_mutation;

#[derive(Interface)]
#[graphql(field(name = "message", type = "String"))]
pub enum Error {
    RegisterUserAlreadyExistsEmailError(RegisterUserAlreadyExistsEmailError),
    RegisterUserInvalidInputError(RegisterUserInvalidInputError),
}
