use async_graphql::{Context, Object, Result};

use crate::{
    database::get_db_pool,
    graphql::{
        auth::get_viewer,
        id_decode,
        models::{
            recruitment::get_recruitment,
            stock::{add_stock, is_already_stocked, remove_stock, Stock},
        },
        mutations::stock_mutation::{
            AddStockAlreadyStockedError, AddStockInput, AddStockResult, AddStockSuccess,
            RemoveStockInput, RemoveStockResult,
        },
    },
};

use super::recruitment_resolver::RecruitmentEdge;

#[derive(Default)]
pub struct StockMutation;

#[Object]
impl StockMutation {
    /// 募集をストックする
    async fn add_stock(&self, ctx: &Context<'_>, input: AddStockInput) -> Result<AddStockResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let decoded_recruitment_id = id_decode(&input.recruitment_id)?;
        // 既にストックしていたらエラーを返す
        if is_already_stocked(pool, viewer.id, decoded_recruitment_id).await? {
            tracing::error!("recruitment is already stocked");
            let error = AddStockAlreadyStockedError {
                message: "recruitment is already stocked".to_string(),
            };
            return Ok(error.into());
        }
        add_stock(pool, viewer.id, decoded_recruitment_id).await?;
        let recruitment = match get_recruitment(pool, decoded_recruitment_id).await? {
            Some(recruitment) => recruitment,
            None => {
                tracing::error!("recruitment not found...");
                return Err(async_graphql::Error::new("recruitment not found..."));
            }
        };
        let success = AddStockSuccess {
            feedback: Stock {
                recruitment_id: recruitment.id,
                viewer_has_stocked: true,
            },
            recruitment_edge: RecruitmentEdge { node: recruitment },
        };

        Ok(success.into())
    }
    /// 募集のストックを外す
    async fn remove_stock(
        &self,
        ctx: &Context<'_>,
        input: RemoveStockInput,
    ) -> Result<RemoveStockResult> {
        let pool = get_db_pool(ctx).await?;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Err(async_graphql::Error::new("Please login")),
        };

        let decoded_recruitment_id = id_decode(&input.recruitment_id)?;
        remove_stock(pool, viewer.id, decoded_recruitment_id).await?;
        let recruitment = match get_recruitment(pool, decoded_recruitment_id).await? {
            Some(recruitment) => recruitment,
            None => {
                tracing::error!("recruitment not found...");
                return Err(async_graphql::Error::new("recruitment not found..."));
            }
        };
        let success = RemoveStockResult {
            feedback: Stock {
                recruitment_id: recruitment.id,
                viewer_has_stocked: false,
            },
        };
        Ok(success)
    }
}
