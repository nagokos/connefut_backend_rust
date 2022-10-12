use anyhow::Result;
use async_graphql::{Object, ID};

use crate::graphql::id_decode;

//* SearchParams */
#[derive(Debug)]
pub struct SearchParams {
    pub use_after: bool, // afterを使用しているかどうか
    pub after: i32,      // decodeしたidを保持
    pub num_rows: i32,   // 何件取得するかを保持
}

impl SearchParams {
    pub fn new(first: Option<i32>, after: Option<ID>) -> Result<Self> {
        if let (Some(first), None) = (first, after.as_ref()) {
            let search_params = SearchParams {
                use_after: false,
                after: 0,
                num_rows: first,
            };
            Ok(search_params)
        } else if let (Some(first), Some(after)) = (first, after.as_ref()) {
            if after.is_empty() {
                return Err(anyhow::anyhow!("afterが正しくありません"));
            }
            let search_params = SearchParams {
                use_after: true,
                after: id_decode(after)? as i32,
                num_rows: first,
            };
            Ok(search_params)
        } else {
            tracing::error!("search params validation error");
            Err(anyhow::anyhow!(
                "[first], [first, after] のいずれかの組み合わせで指定してください"
            ))
        }
    }
}

//* PageInfo */
#[derive(Debug, Clone, Default)]
pub struct PageInfo {
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[Object]
impl PageInfo {
    async fn start_cursor(&self) -> Option<&str> {
        self.start_cursor.as_deref()
    }
    async fn end_cursor(&self) -> Option<&str> {
        self.end_cursor.as_deref()
    }
    async fn has_next_page(&self) -> bool {
        self.has_next_page
    }
    async fn has_previous_page(&self) -> bool {
        self.has_previous_page
    }
}
