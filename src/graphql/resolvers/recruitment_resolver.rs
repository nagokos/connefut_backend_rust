use async_graphql::{Context, Object, Result, ID};
use base64::{encode_config, URL_SAFE};

use crate::{
    database::get_db_pool,
    graphql::{
        auth::get_viewer,
        id_decode,
        models::recruitment::{self, Recruitment},
        mutations::recruitment_mutation::{
            CreateRecruitmentResult, CreateRecruitmentSuccess, RecruitmentInput,
            UpdateRecruitmentResult, UpdateRecruitmentSuccess,
        },
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
    ) -> Result<CreateRecruitmentResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let recruitment = recruitment::create(pool, input, viewer.id).await?;
        let recruitment_edge = RecruitmentEdge {
            cursor: String::default(),
            node: recruitment,
        };
        let success = CreateRecruitmentSuccess { recruitment_edge };
        Ok(success.into())
    }
    async fn update_recruitment(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: RecruitmentInput,
    ) -> Result<UpdateRecruitmentResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let recruitment = recruitment::update(pool, input, id_decode(&id)?, viewer.id).await?;
        let recruitment_edge = RecruitmentEdge {
            cursor: String::default(),
            node: recruitment,
        };
        let success = UpdateRecruitmentSuccess { recruitment_edge };
        Ok(success.into())
    }
}
