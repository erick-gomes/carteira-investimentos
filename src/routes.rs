use crate::{
    AppState,
    controllers::{
        assets_controller,
        auth_controller,
        health_controller,
        html_controller,
        users_controller,
    },
};
use axum::{
    Router,
    routing::{delete, get, patch, post},
};

pub fn create_router() -> Router<AppState> {
    let router = Router::new()
        .route("/", get(html_controller::index))
        .route("/register", get(html_controller::register_form))
        .route("/register", post(html_controller::register))
        .route("/login", get(html_controller::login_form))
        .route("/login", post(html_controller::login))
        .route("/logout", get(html_controller::logout))
        .route("/assets", get(html_controller::assets))
        .nest(
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
