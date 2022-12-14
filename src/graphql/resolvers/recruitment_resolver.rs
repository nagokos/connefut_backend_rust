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

#[derive(Debug, Clone)]
pub struct RecruitmentEdge {
    pub node: Recruitment,
}

#[Object]
impl RecruitmentEdge {
    pub async fn cursor(&self) -> ID {
        id_encode("Recruitment", self.node.id).into()
    }
    pub async fn node(&self) -> Option<Recruitment> {
        self.node.clone().into()
    }
}

#[derive(Default)]
pub struct RecruitmentQuery;

#[Object]
impl RecruitmentQuery {
    /// 公開中の募集のリストを取得する
    async fn recruitments(
        &self,
        ctx: &Context<'_>,
        after: Option<ID>,
        first: Option<i32>,
    ) -> Result<RecruitmentConnection> {
        let pool = get_db_pool(ctx).await?;
        let search_params = SearchParams::new(after, first)?;

        let recruitments = get_recruitments(pool, search_params).await?;

        let edges: Vec<Option<RecruitmentEdge>> = recruitments
            .iter()
            .map(|recruitment| {
                RecruitmentEdge {
                    node: recruitment.to_owned(),
                }
                .into()
            })
            .collect();

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

        Ok(RecruitmentConnection {
            page_info,
            edges: edges.into(),
        })
    }
}

#[derive(Default)]
pub struct RecruitmentMutation;

#[Object]
impl RecruitmentMutation {
    /// 募集を作成する
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
        let recruitment_edge = RecruitmentEdge { node: recruitment };
        let success = CreateRecruitmentSuccess { recruitment_edge };
        Ok(success.into())
    }
    /// 募集の更新をする
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
        let recruitment_edge = RecruitmentEdge { node: recruitment };
        let success = UpdateRecruitmentSuccess { recruitment_edge };
        Ok(success.into())
    }
}
