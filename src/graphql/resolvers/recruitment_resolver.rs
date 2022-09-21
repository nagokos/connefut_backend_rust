use async_graphql::{Context, Object, Result};
use base64::{encode_config, URL_SAFE};

use crate::{
    database::get_db_pool,
    graphql::{
        auth::get_viewer,
        models::recruitment::{self, Recruitment},
        mutations::recruitment_mutation::RecruitmentInput,
    },
};

use super::PageInfo;

#[derive(Debug)]
pub struct RecruitmentConnection {
    pub edges: Option<Vec<Recruitment>>,
    pub page_info: PageInfo,
}

#[Object]
impl RecruitmentConnection {
    pub async fn edges(&self) -> Option<Vec<Recruitment>> {
        self.edges.to_owned()
    }
    pub async fn page_info(&self) -> PageInfo {
        PageInfo {
            start_cursor: None,
            end_cursor: None,
            has_next_page: true,
            has_previous_page: true,
        }
    }
}

#[derive(Debug)]
pub struct RecruitmentEdge {
    pub cursor: String,
    pub node: Recruitment,
}

#[Object]
impl RecruitmentEdge {
    pub async fn cursor(&self) -> String {
        encode_config(format!("Recruitment:{}", self.node.id), URL_SAFE)
    }
    pub async fn node(&self) -> Recruitment {
        self.node.clone()
    }
}

#[derive(Default)]
pub struct RecruitmentMutation;

#[Object]
impl RecruitmentMutation {
    async fn create_recruitment(
        &self,
        ctx: &Context<'_>,
        input: RecruitmentInput,
    ) -> Result<Recruitment> {
        let pool = get_db_pool(ctx).await?;
        let viewer = get_viewer(ctx).await;

        match viewer {
            Some(viewer) => match recruitment::create(pool, input, viewer.id).await {
                Ok(recruitment) => Ok(recruitment),
                Err(e) => Err(e.into()),
            },
            None => Err(async_graphql::Error::new("Please login")),
        }
    }
}
