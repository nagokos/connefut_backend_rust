use std::io::Write;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use connefut_api::graphql::Query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription).finish();
    let mut file = std::fs::File::create("schema.graphql")?;
    let contents = &schema.sdl();
    file.write_all(contents.as_bytes())?;
    Ok(())
}
