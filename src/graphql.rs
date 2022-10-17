use anyhow::Result;
use async_graphql::{EmptySubscription, MergedObject, Schema, ID};
use base64::{decode_config, encode_config, URL_SAFE};

use self::resolvers::{
    prefecture_resolver::PrefectureQuery,
    recruitment_resolver::{RecruitmentMutation, RecruitmentQuery},
    sport_resolver::SportQuery,
    tag_resolver::{TagMutation, TagQuery},
    user_resolver::{UserMutation, UserQuery},
    RootQuery,
};

pub mod auth;
pub mod loader;
pub mod mail;
pub mod models;
pub mod mutations;
pub mod resolvers;
pub mod utils;

#[derive(MergedObject, Default)]
pub struct Query(
    RootQuery,
    PrefectureQuery,
    SportQuery,
    TagQuery,
    UserQuery,
    RecruitmentQuery,
);
#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation, RecruitmentMutation, TagMutation);

pub type GraphqlSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn id_encode(name: &str, id: i64) -> String {
    encode_config(format!("{}:{}", name, id), base64::URL_SAFE)
}

pub fn id_decode(id: &ID) -> Result<i64> {
    let bytes = decode_config(id.as_bytes(), URL_SAFE)?;
    let s = String::from_utf8(bytes)?;
    let split_id: Vec<&str> = s.split(':').collect();
    let decoded_id: i64 = split_id[1].parse()?;
    Ok(decoded_id)
}
