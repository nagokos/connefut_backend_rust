use anyhow::Result;
use async_graphql::{Context, Enum, FieldResult, Object, ID};
use base64::{encode_config, URL_SAFE};
use chrono::{DateTime, Local};
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::graphql::{
    auth::get_viewer,
    id_decode,
    loader::get_loaders,
    mutations::recruitment_mutation::RecruitmentInput,
    utils::pagination::{RecruitmentSearchParams, SearchParams},
};

use super::{
    prefecture::Prefecture,
    sport::Sport,
    tag::{
        add_recruitment_tags, add_recruitment_tags_tx, get_recruitment_tags,
        remove_recruitment_tags_tx, Tag,
    },
    user::User,
};

#[derive(Enum, Clone, Copy, Eq, PartialEq, Debug, sqlx::Type)]
#[sqlx(type_name = "recruitment_category")]
#[sqlx(rename_all = "lowercase")]
pub enum RecruitmentCategory {
    Opponent,
    Personal,
    Member,
    Join,
    Other,
}

#[derive(Enum, Clone, Copy, Eq, PartialEq, Debug, sqlx::Type, Default)]
#[sqlx(type_name = "recruitment_status")]
#[sqlx(rename_all = "lowercase")]
pub enum RecruitmentStatus {
    #[default]
    Draft,
    Published,
    Closed,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Recruitment {
    pub id: i64,
    pub title: String,
    pub category: RecruitmentCategory,
    pub venue: Option<String>,
    pub venue_lat: Option<f64>,
    pub venue_lng: Option<f64>,
    pub start_at: Option<DateTime<Local>>,
    pub closing_at: Option<DateTime<Local>>,
    pub detail: Option<String>,
    pub sport_id: i64,
    pub prefecture_id: i64,
    pub status: RecruitmentStatus,
    pub user_id: i64,
    pub published_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
}

#[Object]
/// 募集
impl Recruitment {
    pub async fn id(&self) -> ID {
        encode_config(format!("Recruitment:{}", self.id), URL_SAFE).into()
    }
    /// 募集のタイトル
    pub async fn title(&self) -> &str {
        &self.title
    }
    /// 募集のカテゴリ
    pub async fn category(&self) -> RecruitmentCategory {
        self.category
    }
    /// 開催場所 募集のカテゴリが対戦相手、個人参加の場合に必要
    pub async fn venue(&self) -> Option<&str> {
        self.venue.as_deref()
    }
    /// 開催場所の緯度
    pub async fn venue_lat(&self) -> Option<f64> {
        self.venue_lat
    }
    /// 開催場所の経度
    pub async fn venue_lng(&self) -> Option<f64> {
        self.venue_lng
    }
    /// 開催日時 募集のカテゴリが対戦相手、個人参加の場合に必要
    pub async fn start_at(&self) -> Option<DateTime<Local>> {
        self.start_at
    }
    /// 募集の掲載期限
    pub async fn closing_at(&self) -> Option<DateTime<Local>> {
        self.closing_at
    }
    /// 募集を公開設定した日時
    pub async fn published_at(&self) -> Option<DateTime<Local>> {
        self.published_at
    }
    /// 募集の詳細
    pub async fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
    /// この募集がストックされている数
    pub async fn stocked_count(&self, ctx: &Context<'_>) -> FieldResult<i64> {
        let loaders = get_loaders(ctx).await;
        let stocked_count = loaders.stock_loader.load_one(self.id).await?;
        match stocked_count {
            Some(count) => Ok(count),
            None => Ok(0),
        }
    }
    /// この募集をログインユーザー(Viewer)がストックしているか
    pub async fn viewer_has_stocked(&self, ctx: &Context<'_>) -> FieldResult<bool> {
        let loaders = get_loaders(ctx).await;
        let viewer = match get_viewer(ctx).await {
            Some(viewer) => viewer,
            None => return Ok(false),
        };

        let viewer_has_stocked = loaders.stock_loader.load_one([viewer.id, self.id]).await?;

        match viewer_has_stocked {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    /// この募集を作成したユーザー
    pub async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<User> {
        let loaders = get_loaders(ctx).await;
        let user = loaders.user_loader.load_one(self.user_id).await?;
        match user {
            Some(user) => Ok(user),
            None => Err(async_graphql::Error::new(String::from("User are a must!"))),
        }
    }
    /// 募集の作成日時
    pub async fn created_at(&self) -> DateTime<Local> {
        self.created_at
    }
    /// 募集のステータス
    pub async fn status(&self) -> RecruitmentStatus {
        self.status
    }
    /// この募集に付与されているタグのリスト
    pub async fn tags(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Tag>> {
        let loaders = get_loaders(ctx).await;
        let tags = loaders.tag_loader.load_one(self.id).await?;
        match tags {
            Some(tags) => Ok(tags),
            None => Ok(Vec::new()), // 募集にタグ紐づいていなかったら配列だけ返す
        }
    }
    /// 募集エリア
    pub async fn prefecture(&self, ctx: &Context<'_>) -> async_graphql::Result<Prefecture> {
        let loaders = get_loaders(ctx).await;
        let prefecture = loaders
            .prefecture_loader
            .load_one(self.prefecture_id)
            .await?;
        match prefecture {
            Some(prefecture) => Ok(prefecture),
            None => Err(async_graphql::Error::new(String::from(
                "Prefecture are a must!",
            ))),
        }
    }
    /// 募集しているスポーツ
    pub async fn sport(&self, ctx: &Context<'_>) -> async_graphql::Result<Sport> {
        let loaders = get_loaders(ctx).await;
        let sport = loaders.sport_loader.load_one(self.sport_id).await?;
        match sport {
            Some(sport) => Ok(sport),
            None => Err(async_graphql::Error::new(String::from("Sport are a must!"))),
        }
    }
}

#[tracing::instrument]
pub async fn get_recruitments(
    pool: &PgPool,
    search_params: SearchParams,
) -> Result<Vec<Recruitment>> {
    let sql = r#"
        SELECT * 
        FROM recruitments
        WHERE ($1 OR id < $2)
        AND status = 'published'
        ORDER BY id DESC
        LIMIT $3
    "#;

    let recruitments = sqlx::query_as::<_, Recruitment>(sql)
        .bind(!search_params.use_after)
        .bind(search_params.after)
        .bind(search_params.num_rows)
        .fetch_all(pool)
        .await;

    match recruitments {
        Ok(recruitments) => {
            tracing::info!("get recruitments successed!!");
            Ok(recruitments)
        }
        Err(e) => {
            tracing::error!("{:?}", e);
            tracing::error!("get recruitments failed...");
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn get_recruitment(pool: &PgPool, id: i64) -> Result<Option<Recruitment>> {
    let sql = "SELECT * FROM recruitments WHERE id = $1";
    let row = sqlx::query_as::<_, Recruitment>(sql)
        .bind(id)
        .fetch_optional(pool)
        .await;

    match row {
        Ok(recruitment) => {
            tracing::info!("get recruitment successed!!");
            Ok(recruitment)
        }
        Err(e) => {
            tracing::error!("get recruitment failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn get_user_recruitments(
    pool: &PgPool,
    params: &RecruitmentSearchParams,
    user_id: i64,
) -> Result<Vec<Recruitment>> {
    let sql = r#"
        SELECT *
        FROM recruitments
        WHERE user_id = $1
        AND ($2 OR status = $3) 
        AND ($4 OR id < $5)
        ORDER BY id DESC
        LIMIT $6
    "#;

    let rows = sqlx::query_as::<_, Recruitment>(sql)
        .bind(user_id)
        .bind(!params.use_status)
        .bind(params.status)
        .bind(!params.use_after)
        .bind(params.after)
        .bind(params.num_rows)
        .fetch_all(pool)
        .await;

    match rows {
        Ok(recruitments) => {
            tracing::info!("get user recruitments successed!!");
            Ok(recruitments)
        }
        Err(e) => {
            tracing::error!("get user recruitments failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn is_next_user_recruitment(
    pool: &PgPool,
    id: i64,
    user_id: i64,
    params: &RecruitmentSearchParams,
) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT id
            FROM recruitments
            WHERE user_id = $1
            AND ($2 OR status = $3)
            AND id < $4
            ORDER BY id DESC
            LIMIT 1
        )
    "#;

    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(!params.use_status)
        .bind(params.status)
        .bind(id)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_exists) => {
            tracing::info!("is next user recruitment successed!!");
            Ok(is_exists)
        }
        Err(e) => {
            tracing::error!("is next user recruitment failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn is_next_recruitment(pool: &PgPool, id: i64) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT id
            FROM recruitments
            WHERE id < $1
            AND status = 'published'
            ORDER BY id DESC
            LIMIT 1
        )
    "#;

    let row = sqlx::query(sql)
        .bind(id)
        .map(|row: PgRow| row.get::<bool, _>(0)) // SELECT EXISTSで true or falseのどちらかが返る
        .fetch_one(pool)
        .await;

    match row {
        Ok(is_next) => {
            tracing::info!("is next recruitment successed!!");
            Ok(is_next)
        }
        Err(e) => {
            tracing::error!("is next recruitment failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn get_stocked_recruitments(
    pool: &PgPool,
    user_id: i64,
    search_params: SearchParams,
) -> Result<Vec<Recruitment>> {
    let sql = r#"
        SELECT r.*
        FROM recruitments as r
        INNER JOIN stocks as s
            ON r.id = s.recruitment_id
        WHERE s.user_id = $1
        AND ($2 OR s.id < ( SELECT id 
                            FROM stocks
                            WHERE user_id = $3
                            AND recruitment_id = $4 )
        )
        AND status = 'published'
        ORDER BY s.id DESC
        LIMIT $5
    "#;

    let rows = sqlx::query_as::<_, Recruitment>(sql)
        .bind(user_id)
        .bind(!search_params.use_after)
        .bind(user_id)
        .bind(search_params.after)
        .bind(search_params.num_rows)
        .fetch_all(pool)
        .await;

    match rows {
        Ok(recruitments) => {
            tracing::info!("get stocked recruitments successed!!");
            Ok(recruitments)
        }
        Err(e) => {
            tracing::error!("get stocked recruitments failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn is_next_stocked_recruitment(pool: &PgPool, id: i64, user_id: i64) -> Result<bool> {
    let sql = r#"
        SELECT EXISTS (
            SELECT *
            FROM recruitments as r
            INNER JOIN stocks as s
                ON r.id = s.recruitment_id
            WHERE s.user_id = $1
            AND s.id < ( SELECT id
                       FROM stocks
                       WHERE user_id = $2
                       AND recruitment_id = $3 )
            AND r.status = 'published'
            ORDER BY s.id DESC
            LIMIT 1
        )
    "#;

    let row = sqlx::query(sql)
        .bind(user_id)
        .bind(user_id)
        .bind(id)
        .map(|row: PgRow| row.get::<bool, _>(0))
        .fetch_one(pool)
        .await;

    match row {
        Ok(has_next) => {
            tracing::info!("is next stocked recruitment successed!!");
            Ok(has_next)
        }
        Err(e) => {
            tracing::error!("is next stocked recruitment failed: {:?}", e);
            Err(e.into())
        }
    }
}

#[tracing::instrument]
pub async fn create(pool: &PgPool, input: RecruitmentInput, user_id: i64) -> Result<Recruitment> {
    let sql = r#"
      INSERT INTO recruitments
        (title, category, venue, venue_lat, venue_lng, start_at, closing_at, 
            detail, sport_id, prefecture_id, status, user_id, published_at, created_at, updated_at)
      VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
      RETURNING *
    "#;

    let now = Local::now();
    let published_at = match input.status {
        RecruitmentStatus::Published => Some(now),
        _ => None,
    };

    let row = sqlx::query_as::<_, Recruitment>(sql)
        .bind(input.title)
        .bind(input.category)
        .bind(input.venue)
        .bind(input.venue_lat)
        .bind(input.venue_lng)
        .bind(input.start_at)
        .bind(input.closing_at)
        .bind(input.detail)
        .bind(id_decode(&input.sport_id)?)
        .bind(id_decode(&input.prefecture_id)?)
        .bind(input.status)
        .bind(user_id)
        .bind(published_at)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await;

    let recruitment = match row {
        Ok(recruitment) => recruitment,
        Err(e) => {
            tracing::error!("create recruitment failed...");
            tracing::error!("{:?}", e.to_string());
            return Err(e.into());
        }
    };

    let decoded_tag_ids = input
        .tag_ids
        .iter()
        .filter_map(|sent_tag| id_decode(sent_tag).ok())
        .collect::<Vec<i64>>();
    add_recruitment_tags(pool, decoded_tag_ids, recruitment.id).await?;
    Ok(recruitment)
}

#[tracing::instrument]
pub async fn update(
    pool: &PgPool,
    input: RecruitmentInput,
    id: i64,
    user_id: i64,
) -> Result<Recruitment> {
    let sql = r#"
        UPDATE recruitments
        SET title = $1, category = $2, venue = $3, venue_lat = $4, venue_lng = $5, start_at = $6,
            closing_at = $7, detail = $8, sport_id = $9, prefecture_id = $10, status = $11, updated_at = $12,
            published_at = CASE
                               WHEN published_at IS NULL THEN $13
                               ELSE published_at
                           END
        WHERE id = $14
        AND user_id = $15
        RETURNING *
    "#;

    let now = Local::now();
    let published_at = match input.status {
        RecruitmentStatus::Published => Some(now),
        _ => None,
    };

    let row = sqlx::query_as::<_, Recruitment>(sql)
        .bind(input.title)
        .bind(input.category)
        .bind(input.venue)
        .bind(input.venue_lat)
        .bind(input.venue_lng)
        .bind(input.start_at)
        .bind(input.closing_at)
        .bind(input.detail)
        .bind(id_decode(&input.sport_id)?)
        .bind(id_decode(&input.prefecture_id)?)
        .bind(input.status)
        .bind(now)
        .bind(published_at)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await;

    let recruitment = match row {
        Ok(recruitment) => {
            tracing::info!("update recruitment successed!");
            recruitment
        }
        Err(e) => {
            tracing::error!("update recruitment failed...");
            tracing::error!("{:?}", e);
            return Err(e.into());
        }
    };

    // ? タグを探す処理これでいいか考え直す
    let current_tags = get_recruitment_tags(pool, recruitment.id).await?; // 募集に不要されているタグを全て取得
    let decoded_sent_tag = input
        .tag_ids
        .iter()
        .filter_map(|sent_tag| id_decode(sent_tag).ok());

    // タグの付与と削除で整合性を保つためにトランザクション
    let mut tx = pool.begin().await?;

    // 送られてきたタグを起点に現在付与されているタグと比較して付与するタグを取得
    let add_tags = decoded_sent_tag
        .clone()
        .into_iter()
        .filter(|sent_tag| {
            !current_tags
                .iter()
                .any(|current_tag| *sent_tag == current_tag.id) // anyは一つでも一致すればtrueを返すため!current_tagsにする
        })
        .collect::<Vec<i64>>();
    if let Err(e) = add_recruitment_tags_tx(&mut tx, add_tags, recruitment.id).await {
        tracing::error!("add_recruitment_tags_tx failed rollback...");
        tx.rollback().await?;
        return Err(e);
    }

    // 現在付与されているタグを起点に送られてきたタグと比較して削除するタグを取得
    let remove_tags = current_tags
        .iter()
        .map(|current_tag| current_tag.id)
        .filter(|&current_tag| {
            !decoded_sent_tag
                .clone()
                .into_iter()
                .any(|sent_tag| sent_tag == current_tag)
        })
        .collect::<Vec<i64>>();
    if let Err(e) = remove_recruitment_tags_tx(&mut tx, remove_tags, recruitment.id).await {
        tracing::error!("remove_recruitment_tags_tx failed rollback...");
        tx.rollback().await?;
        return Err(e);
    }

    // タグの付与、削除に成功したらコミットする
    tx.commit().await?;
    tracing::info!("Transaction Commit!!");
    Ok(recruitment)
}
