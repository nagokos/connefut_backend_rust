use anyhow::Result;
use async_graphql::{Context, EmptySubscription, Guard, MergedObject, Schema, ID};
use async_trait::async_trait;
use base64::{decode_config, encode_config, URL_SAFE};

use self::{
    auth::get_viewer,
    resolvers::{
        prefecture_resolver::PrefectureQuery,
        recruitment_resolver::{RecruitmentMutation, RecruitmentQuery},
        sport_resolver::SportQuery,
        stock_resolver::StockMutation,
        tag_resolver::{TagMutation, TagQuery},
        user_resolver::{UserMutation, UserQuery},
        RootQuery,
    },
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
pub struct Mutation(
    UserMutation,
    RecruitmentMutation,
    TagMutation,
    StockMutation,
);

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

// todo ディレクトリ作ってそこでガード
//* Field Guard */
pub struct FieldGuard {
    pub owner_id: i64,
}

impl FieldGuard {
    fn new(owner_id: i64) -> Self {
        Self { owner_id }
    }
}

// ! async-graphqlのバグでuserにnullが返ってしまう
#[async_trait]
impl Guard for FieldGuard {
    async fn check(&self, ctx: &Context<'_>) -> async_graphql::Result<()> {
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => {
                tracing::error!("You must be logged in to access this field");
                return Err(async_graphql::Error::new(
                    "You must be logged in to access this field",
                ));
            }
        };

        if viewer.id != self.owner_id {
            tracing::error!("This field is not accessible");
            return Err(async_graphql::Error::new("This field is not accessible"));
        };

        Ok(())
    }
}
