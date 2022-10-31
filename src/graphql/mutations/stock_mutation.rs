use async_graphql::{InputObject, SimpleObject, Union, ID};

use crate::graphql::{
    models::recruitment::Recruitment, resolvers::recruitment_resolver::RecruitmentEdge,
};

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

#[derive(SimpleObject)]
pub struct RemoveStockResult {
    pub recruitment: Recruitment,
}
