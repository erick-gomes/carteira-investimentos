use sqlx::PgPool;

pub mod controllers;
pub mod routes;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
}
