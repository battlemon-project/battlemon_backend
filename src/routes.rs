use crate::graphql::QueryRoot;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use axum::extract::FromRef;
use axum::routing::get;
use axum::Router;
use graphql::*;
use healthcheck::*;
use sqlx::PgPool;

mod graphql;
mod healthcheck;

pub type BattlemonSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub graphql: BattlemonSchema,
}

impl FromRef<AppState> for BattlemonSchema {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.graphql.clone()
    }
}

#[rustfmt::skip]
#[tracing::instrument(name = "Setup routes", skip_all)]
pub fn setup_router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/graphql/playground",get(graphql_playground).post(graphql_handler))
        .with_state(state)
}
