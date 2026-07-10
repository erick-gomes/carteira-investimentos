use axum::{Json, http::StatusCode};
use sqlx::PgPool;

pub mod controllers;
pub mod errors;
pub mod extractors;
pub mod models;
pub mod routes;
pub mod utils;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
}

pub type Response<T> = Result<(StatusCode, Json<T>), errors::AppError>;
