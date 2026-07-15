use crate::{
    AppState,
    controllers::{assets_controller, auth_controller, health_controller, users_controller},
};
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn create_router() -> Router<AppState> {
    let router = Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/health", get(health_controller::health_check))
            .route("/users", post(users_controller::create_user))
            .route("/auth/login", post(auth_controller::authenticate_user))
            .route("/auth/refresh", post(auth_controller::refresh_access_token))
            .route("/user/me", get(users_controller::get_user_me))
            .route("/assets", get(assets_controller::get_all_assets))
            .route("/assets", post(assets_controller::create_asset))
            .route("/assets/{id}", patch(assets_controller::patch_asset))
            .route("/assets/{id}", delete(assets_controller::delete_asset)),
    );
    router
}
