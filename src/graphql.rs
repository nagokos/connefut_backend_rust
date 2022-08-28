use async_graphql::{EmptyMutation, EmptySubscription, MergedObject, Schema};
use thiserror::Error;

use self::resolvers::{
    prefecture_resolver::PrefectureQuery, sport_resolver::SportQuery, tag_resolver::TagQuery,
};

pub mod models;
pub mod resolvers;

#[derive(MergedObject, Default)]
pub struct Query(PrefectureQuery, SportQuery, TagQuery);

pub type GraphqlSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Internal Server Error")]
    ServerError(String),
}
