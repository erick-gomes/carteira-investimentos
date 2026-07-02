use crate::{
    AppState,
    controllers::{health_controller, users_controller},
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn create_router() -> Router<AppState> {
    let router = Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/health", get(health_controller::health_check))
            .route("/users", post(users_controller::create_user)),
    );
    router
}
