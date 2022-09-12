use std::io::Write;

use async_graphql::{EmptySubscription, Schema};
use connefut_api::graphql::{
    mutations::{
        user_mutation::{LoginUserInput, LoginUserResult, RegisterUserInput, RegisterUserResult},
        Error,
    },
    resolvers::Node,
    Mutation, Query,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .register_output_type::<Node>()
        .register_output_type::<Error>()
        .register_output_type::<RegisterUserResult>()
        .register_output_type::<LoginUserResult>()
        .register_input_type::<RegisterUserInput>()
        .register_input_type::<LoginUserInput>()
        .finish();
    let mut file = std::fs::File::create("schema.graphql")?;
    let contents = &schema.sdl();
    file.write_all(contents.as_bytes())?;
    Ok(())
}
