use std::io::Write;

use async_graphql::{EmptySubscription, Schema};
use connefut_api::graphql::{
    mutations::{
        recruitment_mutation::RecruitmentInput,
        stock_mutation::{AddStockInput, AddStockResult, RemoveStockInput, RemoveStockResult},
        tag_mutation::{CreateTagInput, CreateTagResult},
        user_mutation::{LoginUserInput, LoginUserResult, RegisterUserInput, RegisterUserResult},
        Error,
    },
    resolvers::{
        recruitment_resolver::{RecruitmentConnection, RecruitmentEdge},
        tag_resolver::TagEdge,
        Node,
    },
    utils::pagination::PageInfo,
    Mutation, Query,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .register_output_type::<Node>()
        .register_output_type::<Error>()
        .register_output_type::<RegisterUserResult>()
        .register_output_type::<LoginUserResult>()
        .register_output_type::<CreateTagResult>()
        .register_output_type::<PageInfo>()
        .register_output_type::<RecruitmentConnection>()
        .register_output_type::<RecruitmentEdge>()
        .register_output_type::<TagEdge>()
        .register_output_type::<AddStockResult>()
        .register_output_type::<RemoveStockResult>()
        .register_input_type::<RegisterUserInput>()
        .register_input_type::<LoginUserInput>()
        .register_input_type::<RecruitmentInput>()
        .register_input_type::<CreateTagInput>()
        .register_input_type::<AddStockInput>()
        .register_input_type::<RemoveStockInput>()
        .finish();
    let mut file = std::fs::File::create("schema.graphql")?;
    let contents = &schema.sdl();
    file.write_all(contents.as_bytes())?;
    Ok(())
}
