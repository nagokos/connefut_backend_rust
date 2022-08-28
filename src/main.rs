use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    http::{HeaderValue, Method},
    routing::post,
    Extension, Router, Server,
};
use connefut_api::graphql::{GraphqlSchema, Query};
use std::{sync::Arc, *};
use tracing_subscriber::fmt::format::FmtSpan;

use tower_http::cors::CorsLayer;

mod database;
pub mod graphql;
use database::pool;

async fn graphql_handler(schema: Extension<GraphqlSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let server = async {
        let pool = pool().await.unwrap();
        let arc_pool = Arc::new(pool);
        let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
            .data(arc_pool)
            .finish();

        let app = Router::new()
            .route("/graphql", post(graphql_handler))
            .layer(
                CorsLayer::new()
                    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                    .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
                    .allow_credentials(true),
            )
            .layer(Extension(schema));

        Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    };
    tracing::info!("start graphql server");
    tokio::join!(server);
}
