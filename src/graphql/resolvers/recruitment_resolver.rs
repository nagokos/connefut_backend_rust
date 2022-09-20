use async_graphql::Object;
use base64::{encode_config, URL_SAFE};

use crate::graphql::models::recruitment::Recruitment;

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
