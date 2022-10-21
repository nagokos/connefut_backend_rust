use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use sqlx::{Pool, Postgres};

use self::{
    prefecture::PrefectureLoader, sport::SportLoader, stock::StockLoader, tag::TagLoader,
    user::UserLoader,
};

pub mod prefecture;
pub mod sport;
pub mod stock;
pub mod tag;
pub mod user;

pub struct Loaders {
    pub user_loader: DataLoader<UserLoader>,
    pub tag_loader: DataLoader<TagLoader>,
    pub prefecture_loader: DataLoader<PrefectureLoader>,
    pub sport_loader: DataLoader<SportLoader>,
    pub stock_loader: DataLoader<StockLoader>,
}

impl Loaders {
    pub fn new(pool: &Arc<Pool<Postgres>>) -> Self {
        let user_loader = DataLoader::new(UserLoader { pool: pool.clone() }, tokio::spawn);
        let tag_loader = DataLoader::new(TagLoader { pool: pool.clone() }, tokio::spawn);
        let prefecture_loader =
            DataLoader::new(PrefectureLoader { pool: pool.clone() }, tokio::spawn);
        let sport_loader = DataLoader::new(SportLoader { pool: pool.clone() }, tokio::spawn);
        let stock_loader = DataLoader::new(StockLoader { pool: pool.clone() }, tokio::spawn);

        Self {
            user_loader,
            tag_loader,
            prefecture_loader,
            sport_loader,
            stock_loader,
        }
    }
}
