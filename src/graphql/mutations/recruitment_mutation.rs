use async_graphql::{Enum, InputObject, SimpleObject, Union, ID};
use chrono::{DateTime, Local};
use validator::Validate;

use crate::graphql::models::recruitment::{Category, Recruitment, Status};

#[derive(InputObject, Debug, Validate)]
pub struct RecruitmentInput {
    #[validate(length(max = 60, message = "タイトルは60文字以内にしてください"))]
    pub title: String,
    pub sport_id: ID,
    pub category: Category,
    #[validate(length(max = 10000, message = "募集の詳細は10000文字以内で入力してください"))]
    pub detail: Option<String>,
    pub prefecture_id: ID,
    pub venue: Option<String>,
    pub venue_lat: Option<f64>,
    pub venue_lng: Option<f64>,
    pub start_at: Option<DateTime<Local>>,
    pub closing_at: Option<DateTime<Local>>,
    pub status: Status,
    pub tag_ids: Vec<ID>,
}

#[derive(Union)]
pub enum CreateRecruitmentResult {
    CreateRecruitmentSuccess(CreateRecruitmentSuccess),
    CreateRecruitmentInvalidInputErrors(CreateRecruitmentInvalidInputErrors),
}

#[derive(SimpleObject, Debug)]
pub struct CreateRecruitmentSuccess {
    pub recruitment: Recruitment,
}

#[derive(SimpleObject, Debug)]
pub struct CreateRecruitmentInvalidInputErrors {
    pub errors: Vec<CreateRecruitmentInvalidInputError>,
}

#[derive(SimpleObject, Debug)]
pub struct CreateRecruitmentInvalidInputError {
    pub message: String,
    pub field: RecruitmentInvalidInputField,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum RecruitmentInvalidInputField {
    Title,
    SportId,
    Category,
    Detail,
    PrefectureId,
    Venue,
    VenueLat,
    VenutLng,
    StartAt,
    ClosingAt,
}
