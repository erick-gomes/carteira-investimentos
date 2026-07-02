use axum::{Json, extract::State};
use serde::Deserialize;
use tracing::instrument;

use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    username: String,
    email: String,
    password: String,
}

#[instrument]
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUser>,
) -> &'static str {
    println!("Creating user: {:?}", body.email);
    println!("Creating user: {:?}", body.username);
    println!("Creating user: {:?}", body.password);
    "User created successfully"
}
