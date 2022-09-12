use async_graphql::Context;

use crate::graphql::models::user::User;

pub mod cookie;
pub mod jwt;

pub async fn get_viewer<'ctx>(ctx: &Context<'ctx>) -> &'ctx Option<User> {
    match ctx.data_opt::<Option<User>>() {
        Some(viewer) => viewer,
        None => &None,
    }
}
