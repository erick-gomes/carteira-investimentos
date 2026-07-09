use crate::database::errors::PostgresError;
use crate::errors::AppError;
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
    #[validate(length(
        min = 3,
        max = 50,
        message = "O nome de usuário deve ter entre 3 e 50 caracteres"
    ))]
    username: String,
    #[validate(email(message = "O formato do e-mail informado é inválido"))]
    email: String,
    #[validate(length(
        min = 6,
        max = 100,
        message = "A senha deve ter entre 6 e 100 caracteres"
    ))]
    password: String,
}

#[derive(Serialize)]
pub struct CreateUserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

#[instrument(skip(state, body), fields(username = %body.username, email = %body.email))]
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateUserRequest>,
) -> Response<CreateUserResponse> {
    let email_normalizado = body.email.to_lowercase().trim().to_string();
    let username_normalizado = body.username.trim().to_string();

    if username_normalizado.is_empty() || email_normalizado.is_empty() {
        return Err(AppError::BadRequest(
            "Nome de usuário e e-mail não podem ser vazios".to_string(),
        ));
    }

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
        username: username_normalizado,
        email: email_normalizado,
        password_hash,
    };
    let user = users_model::create_user(&state.pool, user_model)
        .await
        .map_err(|error| {
            tracing::error!("Erro ao criar usuário: {:?}", error);
            let db_error_code = error
                .as_database_error()
                .and_then(|db_err| db_err.code())
                .map(|code| PostgresError::from(code.as_ref()));
            match db_error_code {
                Some(PostgresError::UniqueViolation) => AppError::Conflict(
                    "O e-mail ou nome de usuário informado já está em uso".to_string(),
                ),
                _ => AppError::Database(error),
            }
        })?;
    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }),
    ))
}
