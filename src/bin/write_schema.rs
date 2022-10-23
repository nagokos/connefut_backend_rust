use std::io::Write;

use async_graphql::{EmptySubscription, Schema};
use connefut_api::graphql::{mutations::Error, resolvers::Node, Mutation, Query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .register_output_type::<Node>()
        .register_output_type::<Error>()
        .finish();
    let mut file = std::fs::File::create("schema.graphql")?;
    let contents = &schema.sdl();
    file.write_all(contents.as_bytes())?;
    Ok(())
}
