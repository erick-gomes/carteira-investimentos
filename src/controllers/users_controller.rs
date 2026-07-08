use crate::extractors::ValidatedJson;
use crate::models::users_model::{self, CreateUserModel};
use crate::{AppState, Response};
use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6, max = 100))]
    password: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[instrument(skip(state, body), fields(username = %body.username, email = %body.email))]
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateUserRequest>,
) -> Response<CreateUserResponse> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|error| {
            tracing::error!("Erro ao gerar hash da senha: {:?}", error);
            anyhow!("Erro ao gerar hash da senha")
        })?
        .to_string();
    let user_model = CreateUserModel {
        username: body.username,
        email: body.email,
        password_hash,
    };
    let user = users_model::create_user(&state.pool, user_model).await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}
