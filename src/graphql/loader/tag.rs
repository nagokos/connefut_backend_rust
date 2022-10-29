use std::{collections::HashMap, sync::Arc};

use async_graphql::{dataloader::Loader, futures_util::TryStreamExt};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::graphql::models::tag::Tag;

pub struct TagLoader {
    pub pool: Arc<PgPool>,
}

#[async_trait]
impl Loader<i64> for TagLoader {
    type Value = Vec<Tag>;
    type Error = Arc<sqlx::Error>;

    // 募集に紐づいているタグを取得する 募集のIDからタグを複数
    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let sql = r#"
            SELECT t.id, t.name, r_t.recruitment_id
            FROM tags as t
            INNER JOIN recruitment_tags as r_t
            ON t.id = r_t.tag_id
            WHERE r_t.recruitment_id IN (
        "#;

        let mut query_builder = QueryBuilder::<Postgres>::new(sql);
        let mut separated = query_builder.separated(", ");
        for key in keys.iter() {
            separated.push_bind(key);
        }
        separated.push_unseparated(") ");

        let query = query_builder.build();
        let mut rows = query.fetch(&*self.pool);

        // { recruitment_id: [tag] }の形になるように整形
        let mut recruitment_tags_hash: HashMap<i64, Vec<Tag>> = HashMap::new();
        while let Some(row) = rows.try_next().await? {
            let id: i64 = row.get("id");
            let name: &str = row.get("name");
            let tag = Tag {
                id,
                name: name.to_string(),
            };
            let recruitment_id: i64 = row.get("recruitment_id");
            if let Some(current_tags) = recruitment_tags_hash.get_mut(&recruitment_id) {
                current_tags.push(tag);
            } else {
                recruitment_tags_hash.insert(recruitment_id, vec![tag]);
            };
        }
        Ok(recruitment_tags_hash)
    }
}
