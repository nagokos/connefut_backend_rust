use async_graphql::{InputObject, Object, SimpleObject, Union, ID};

use crate::graphql::{
    id_encode, models::stock::Stock, resolvers::recruitment_resolver::RecruitmentEdge,
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
    async fn remove_recruitment_id(&self) -> ID {
        // relayでストック一覧から削除する際に必要
        id_encode("Recruitment", self.feedback.recruitment_id).into()
    }
}
