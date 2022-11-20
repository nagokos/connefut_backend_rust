use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    http::{
        header::{self, HeaderMap},
        HeaderValue, Method,
    },
    routing::{get, post},
    Extension, Router, Server,
};
use sqlx::PgPool;
use std::{sync::Arc, *};
use tower_http::cors::CorsLayer;
use tracing_subscriber::fmt::format::FmtSpan;

use self::graphql::loader::Loaders;
use self::graphql::{GraphqlSchema, Mutation, Query};
use crate::graphql::auth::{
    cookie::get_value_from_cookie,
    external::{
        google::{auth_google_callback, auth_google_redirect, new_google_auth_client},
        line::{auth_line_callback, auth_line_redirect, new_line_auth_client},
    },
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
    if let Some(token) = get_value_from_cookie(&headers, "token") {
        let user = get_user_from_token(&pool, token).await;
        // ctx.data::<Option<User>>でログインユーザにアクセスできる
        req = req.data(user);
    }

    schema.execute(req).await.into()
}

// todo unwrap使わない
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let server = async {
        let config = get_config();
        let pool = pool(config).await.unwrap();
        let pool = Arc::new(pool);
        let google_auth_client = Arc::new(new_google_auth_client(&config.google).await.unwrap());
        let line_auth_client = Arc::new(new_line_auth_client(&config.line).await.unwrap());
        let loaders = Loaders::new(&pool);
        let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
            .data(Arc::clone(&pool))
            .data(loaders)
            .data(config)
            .finish();

        let app = Router::new()
            .route("/graphql", post(graphql_handler))
            .route("/oauth/google", get(auth_google_redirect))
            .route("/oauth/google/callback", get(auth_google_callback))
            .route("/oauth/line", get(auth_line_redirect))
            .route("/oauth/line/callback", get(auth_line_callback))
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
            .layer(Extension(config))
            .layer(Extension(schema))
            .layer(Extension(google_auth_client))
            .layer(Extension(line_auth_client))
            .layer(Extension(pool));

        Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    };
    tracing::info!("start graphql server");
    tokio::join!(server);
}
