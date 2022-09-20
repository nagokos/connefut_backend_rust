use async_graphql::{InputObject, ID};
use chrono::{DateTime, Local};
use validator::Validate;

use crate::graphql::models::recruitment::{Category, Status};

#[derive(InputObject, Debug, Validate)]
pub struct RecruitmentInput {
    pub title: String,
    pub sport_id: ID,
    pub category: Category,
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

//#[derive(Union)]
//pub enum CreateRecruitmentResult {}

//#[derive(SimpleObject, Debug)]
//pub struct CreateRecruitmentSuccess {}
