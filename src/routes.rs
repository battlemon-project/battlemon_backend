use axum::routing::get;
use axum::Router;
mod healthcheck;

use healthcheck::*;

pub fn get_router() -> Router {
    Router::new().route("/healthcheck", get(healthcheck))
}
