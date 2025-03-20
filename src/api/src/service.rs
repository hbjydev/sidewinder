use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_macros::FromRef;
use sidewinder_core::exec::driver::Driver;

use crate::error::Result;

#[derive(Clone, FromRef)]
pub struct ApiState {
    pub driver: Arc<dyn Driver>,
}

pub fn new_router(driver: Arc<dyn Driver>) -> Router {
    let state = ApiState { driver };

    Router::new()
        .route("/health", get(route_healthcheck))
        .with_state(state)
}

#[axum::debug_handler]
async fn route_healthcheck(
    State(_s): State<ApiState>,
) -> Result<impl IntoResponse> {
    Ok((StatusCode::OK, "OK"))
}