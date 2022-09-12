use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    http::{
        header::{self, HeaderMap},
        HeaderValue, Method,
    },
    routing::post,
    Extension, Router, Server,
};
use sqlx::PgPool;
use std::{sync::Arc, *};
use tower_http::cors::CorsLayer;
use tracing_subscriber::fmt::format::FmtSpan;

use self::graphql::{GraphqlSchema, Mutation, Query};
use crate::graphql::auth::{
    cookie::{get_cookie_from_header, get_value_from_cookie},
    jwt::get_user_from_token,
};
pub mod config;
mod database;
mod graphql;

use config::get_config;
use database::pool;

async fn graphql_handler(
    Extension(schema): Extension<GraphqlSchema>,
    req: GraphQLRequest,
    headers: HeaderMap,
    Extension(pool): Extension<Arc<PgPool>>,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(cookie) = get_cookie_from_header(&headers) {
        if let Some(token) = get_value_from_cookie(cookie, "token") {
            let user = get_user_from_token(&pool, token).await;
            // ctx.data::<Option<User>>でログインユーザにアクセスできる
            req = req.data(user);
        }
    }

    schema.execute(req).await.into()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let server = async {
        let config = get_config();
        let pool = pool(config).await.unwrap();
        let arc_pool = Arc::new(pool);
        let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
            .data(arc_pool.clone())
            .data(config)
            .finish();

        let app = Router::new()
            .route("/graphql", post(graphql_handler))
            .layer(
                CorsLayer::new()
                    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                    .allow_headers(vec![
                        header::ACCEPT,
                        header::ACCEPT_LANGUAGE,
                        header::AUTHORIZATION,
                        header::CONTENT_LANGUAGE,
                        header::CONTENT_TYPE,
                    ])
                    .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                    .allow_credentials(true),
            )
            .layer(Extension(schema))
            .layer(Extension(arc_pool));

        Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    };
    tracing::info!("start graphql server");
    tokio::join!(server);
}
