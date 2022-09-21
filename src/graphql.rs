use anyhow::Result;
use async_graphql::{EmptySubscription, MergedObject, Schema, ID};
use base64::{decode_config, URL_SAFE};

use self::resolvers::{
    prefecture_resolver::PrefectureQuery,
    recruitment_resolver::RecruitmentMutation,
    sport_resolver::SportQuery,
    tag_resolver::TagQuery,
    user_resolver::{UserMutation, UserQuery},
    RootQuery,
};

pub mod auth;
pub mod mail;
pub mod models;
pub mod mutations;
pub mod resolvers;

#[derive(MergedObject, Default)]
pub struct Query(RootQuery, PrefectureQuery, SportQuery, TagQuery, UserQuery);
#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation, RecruitmentMutation);

pub type GraphqlSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn id_decode(id: &ID) -> Result<i64> {
    let bytes = decode_config(id.as_bytes(), URL_SAFE)?;
    let s = String::from_utf8(bytes)?;
    let split_id: Vec<&str> = s.split(':').collect();
    let decoded_id: i64 = split_id[1].parse()?;
    Ok(decoded_id)
}

pub fn ids_decode(ids: &Vec<ID>) -> Result<Vec<i64>> {
    let decoded_ids = ids
        .iter()
        .filter_map(|id| id_decode(id).ok())
        .collect::<Vec<i64>>();
    Ok(decoded_ids)
}
