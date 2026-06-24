use crate::controllers::health_controller::HealthController;
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new().route("/health", get(HealthController::health_check)),
    )
}
