use async_graphql::{InputObject, Object, SimpleObject, Union, ID};

use crate::graphql::{models::stock::Stock, resolvers::recruitment_resolver::RecruitmentEdge};

// Add Stock Mutation
#[derive(InputObject)]
pub struct AddStockInput {
    pub recruitment_id: ID,
}

#[derive(Union)]
#[allow(clippy::large_enum_variant)]
pub enum AddStockResult {
    AddStockSuccess(AddStockSuccess),
    AddStockAlreadyStockedError(AddStockAlreadyStockedError),
}

#[derive(SimpleObject)]
pub struct AddStockSuccess {
    pub feedback: Stock,
    pub recruitment_edge: RecruitmentEdge,
}

#[derive(SimpleObject)]
pub struct AddStockAlreadyStockedError {
    pub message: String,
}

// Remove Stock Mutation
#[derive(InputObject)]
pub struct RemoveStockInput {
    pub recruitment_id: ID,
}

pub struct RemoveStockResult {
    pub feedback: Stock,
}

#[Object]
impl RemoveStockResult {
    async fn feedback(&self) -> Stock {
        self.feedback
    }
}
