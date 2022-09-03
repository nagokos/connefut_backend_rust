use async_graphql::{EmptySubscription, MergedObject, Schema};
use thiserror::Error;

use self::resolvers::{
    prefecture_resolver::PrefectureQuery,
    sport_resolver::SportQuery,
    tag_resolver::TagQuery,
    user_resolver::{UserMutation, UserQuery},
    RootQuery,
};

pub mod models;
pub mod mutations;
pub mod resolvers;

#[derive(MergedObject, Default)]
pub struct Query(RootQuery, PrefectureQuery, SportQuery, TagQuery, UserQuery);
#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation);

pub type GraphqlSchema = Schema<Query, Mutation, EmptySubscription>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Internal Server Error")]
    ServerError(String),
}
