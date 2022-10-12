use anyhow::Result;
use chrono::{Duration, Local};
use connefut_api::{
    config::get_config,
    database::pool,
    graphql::models::{
        prefecture::Prefecture,
        recruitment::{Category, Recruitment, Status},
        sport::Sport,
        user::User,
    },
};
use rand::Rng;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::ops::Add;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let config = get_config();
    let pool = pool(config).await?;
    create_recruitments(&pool).await?;
    tracing::info!("create test recruitments data!!");
    Ok(())
}

async fn get_users(pool: &PgPool) -> Result<Vec<User>> {
    let sql = "SELECT * FROM users";
    let users = sqlx::query_as::<_, User>(sql).fetch_all(pool).await?;
    Ok(users)
}

async fn get_sports(pool: &PgPool) -> Result<Vec<Sport>> {
    let sql = "SELECT * FROM sports";
    let sports = sqlx::query_as::<_, Sport>(sql).fetch_all(pool).await?;
    Ok(sports)
}

async fn get_prefectures(pool: &PgPool) -> Result<Vec<Prefecture>> {
    let sql = "SELECT * FROM prefectures";
    let prefectures = sqlx::query_as::<_, Prefecture>(sql).fetch_all(pool).await?;
    Ok(prefectures)
}

#[tracing::instrument]
async fn create_recruitments(pool: &PgPool) -> Result<()> {
    let detail = r#"
        東京都社会人3部リーグに所属しているFortuna TOKYOと申します。
        下記の通りグラウンドが取得できましたので、対戦相手の募集をいたします。
        ※先着順ではございません。
        ※他でも打診をしております。
        応募を多数いただく場合はチーム内協議の上決定いたします。
        
        日時:4月16日（土）8:30〜10:30
        場所:朝霞中央公園陸上競技場(綺麗な人工芝)
        費用:6000円
        
        〈募集条件〉
        ①暴力、暴言、ラフプレーなどが無いよう、リスペクトの精神を持ってプレーできる事
        ②対戦決定後キャンセルしない事
        ③当日審判、グラウンドの準備、整備にご協力頂ける事
        ④13人以上揃う事
        ⑤競技思考である事
        ⑥コロナ感染対策にご協力いただける事
        
        ◆当チームプロフィール◆
        チーム名  Fortuna TOKYO
        ユニフォーム色 青 or 赤
        平均年齢  27
        
        対戦をご希望される方は、
        チーム名：
        代表者名：
        代表者電話番号：
        ユニフォーム色：
        所属リーグ等チーム情報：
        
        上記ご記入の上ご連絡ください。
        
        以上、よろしくお願いいたします。
    "#;
    let now = Local::now();
    let users = get_users(pool).await?;
    let sports = get_sports(pool).await?;
    let prefectures = get_prefectures(pool).await?;
    let mut rng = rand::thread_rng();
    let recruitments = (0..=1000).map(|i| Recruitment {
        id: i,
        title: format!("対戦相手募集 朝霞中央公園陸上競技場(人工芝) {}", i),
        category: Category::Opponent,
        venue: Some(String::from("朝霞中央公園陸上競技場")),
        start_at: Some(now.add(Duration::days(2))),
        closing_at: Some(now.add(Duration::days(1))),
        detail: Some(detail.to_string()),
        venue_lat: None,
        venue_lng: None,
        status: Status::Published,
        published_at: Some(now),
        created_at: now,
        user_id: users.get(rng.gen_range(0..users.len())).unwrap().id,
        sport_id: sports.get(rng.gen_range(0..sports.len())).unwrap().id,
        prefecture_id: prefectures
            .get(rng.gen_range(0..prefectures.len()))
            .unwrap()
            .id,
    });

    let sql = r#"
        INSERT INTO recruitments
        (title, category, venue, venue_lat, venue_lng, start_at, closing_at, 
            detail, sport_id, prefecture_id, status, user_id, published_at, created_at, updated_at)
    "#;
    let mut query_builder = QueryBuilder::<Postgres>::new(sql);
    query_builder.push_values(recruitments, |mut b, r| {
        b.push_bind(r.title)
            .push_bind(r.category)
            .push_bind(r.venue)
            .push_bind(r.venue_lat)
            .push_bind(r.venue_lng)
            .push_bind(r.start_at)
            .push_bind(r.closing_at)
            .push_bind(r.detail)
            .push_bind(r.sport_id)
            .push_bind(r.prefecture_id)
            .push_bind(r.status)
            .push_bind(r.user_id)
            .push_bind(r.published_at)
            .push_bind(r.created_at)
            .push_bind(now);
    });
    let query = query_builder.build();
    query.execute(pool).await?;
    Ok(())
}
