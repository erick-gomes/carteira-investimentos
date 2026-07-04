use sqlx::PgPool;

pub mod controllers;
pub mod models;
pub mod routes;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
}
