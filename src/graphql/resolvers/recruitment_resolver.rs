use async_graphql::{Context, Object, Result, SimpleObject, ID};
use base64::encode_config;

use crate::{
    database::get_db_pool,
    graphql::{
        auth::get_viewer,
        id_decode, id_encode,
        models::recruitment::{self, get_recruitments, is_next_recruitment, Recruitment},
        mutations::recruitment_mutation::{
            CreateRecruitmentResult, CreateRecruitmentSuccess, RecruitmentInput,
            UpdateRecruitmentResult, UpdateRecruitmentSuccess,
        },
        utils::pagination::{PageInfo, SearchParams},
    },
};

#[derive(SimpleObject, Debug)]
pub struct RecruitmentConnection {
    pub edges: Option<Vec<Option<RecruitmentEdge>>>,
    pub page_info: PageInfo,
}

#[Object]
impl RecruitmentConnection {
    pub async fn edges(&self) -> Option<Vec<RecruitmentEdge>> {
        self.edges.clone()
    }
    pub async fn page_info(&self) -> PageInfo {
        self.page_info.clone()
    }
}

#[derive(Debug, Clone)]
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
pub struct RecruitmentQuery;

#[Object]
impl RecruitmentQuery {
    async fn recruitments(
        &self,
        ctx: &Context<'_>,
        after: Option<ID>,
        first: Option<i32>,
    ) -> Result<RecruitmentConnection> {
        let pool = get_db_pool(ctx).await?;
        let search_params = SearchParams::new(first, after)?;

        let recruitments = get_recruitments(pool, search_params).await?;

        let edges = if recruitments.is_empty() {
            None
        } else {
            let edges = recruitments
                .iter()
                .map(|recruitment| RecruitmentEdge {
                    cursor: String::default(),
                    node: recruitment.to_owned(),
                })
                .collect::<Vec<RecruitmentEdge>>();
            Some(edges)
        };

        let page_info = match recruitments.last() {
            Some(recruitment) => {
                let is_next = is_next_recruitment(pool, recruitment.id).await?;
                let encoded_id =
                    encode_config(format!("Recruitment:{}", recruitment.id), base64::URL_SAFE);
                PageInfo {
                    has_next_page: is_next,
                    end_cursor: Some(encoded_id),
                    ..Default::default()
                }
            }
            None => Default::default(),
        };

        Ok(RecruitmentConnection { page_info, edges })
    }
    async fn viewer_recruitments(
        &self,
        ctx: &Context<'_>,
        after: Option<ID>,
        first: Option<i32>,
    ) -> Result<RecruitmentConnection> {
        let pool = get_db_pool(ctx).await?;
        let search_params = SearchParams::new(first, after)?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let recruitments = get_viewer_recruitments(pool, search_params, viewer.id).await?;

        let edges = if recruitments.is_empty() {
            None
        } else {
            let edges = recruitments
                .iter()
                .map(|recruitment| RecruitmentEdge {
                    cursor: String::default(),
                    node: recruitment.to_owned(),
                })
                .collect::<Vec<RecruitmentEdge>>();
            Some(edges)
        };

        let page_info = match recruitments.last() {
            Some(recruitment) => {
                let is_next = is_next_viewer_recruitment(pool, recruitment.id, viewer.id).await?;
                let encoded_id =
                    encode_config(format!("Recruitment:{}", recruitment.id), base64::URL_SAFE);
                PageInfo {
                    has_next_page: is_next,
                    end_cursor: Some(encoded_id),
                    ..Default::default()
                }
            }
            None => Default::default(),
        };

        Ok(RecruitmentConnection { edges, page_info })
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
