use anyhow::Result;
use async_graphql::{InputObject, SimpleObject, Union};
use sqlx::PgPool;

use crate::graphql::{models::tag::is_already_exists_tag_name, resolvers::tag_resolver::TagEdge};

#[derive(InputObject)]
pub struct CreateTagInput {
    pub name: String,
}

impl CreateTagInput {
    pub async fn validate_is_already_tag_name(
        &self,
        pool: &PgPool,
    ) -> Result<Option<CreateTagAlreadyExistsNameError>> {
        let is_exists = is_already_exists_tag_name(pool, &self.name).await?;
        if is_exists {
            tracing::error!("This tag name already exists");
            let error = CreateTagAlreadyExistsNameError {
                message: String::from("このタグは既に存在します"),
            };
            return Ok(Some(error));
        }
        Ok(None)
    }
}

#[derive(Union)]
pub enum CreateTagResult {
    CreateTagSuccess(CreateTagSuccess),
    CreateTagAlreadyExistsNameError(CreateTagAlreadyExistsNameError),
}

#[derive(SimpleObject, Debug)]
pub struct CreateTagSuccess {
    pub tag_edge: TagEdge,
}

#[derive(SimpleObject, Debug)]
pub struct CreateTagAlreadyExistsNameError {
    pub message: String,
}
