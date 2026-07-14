use crate::{
    AppState,
    controllers::{auth_controller, health_controller, users_controller},
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
            .route("/users", post(users_controller::create_user))
            .route("/auth/login", post(auth_controller::authenticate_user))
            .route("/auth/refresh", post(auth_controller::refresh_access_token))
            .route("/user/me", get(users_controller::get_user_me)),
    );
    router
}
