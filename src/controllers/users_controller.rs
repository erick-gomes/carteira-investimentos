use crate::errors::AppError;
use crate::errors::PostgresError;
use crate::extractors::AuthenticatedUser;
use crate::extractors::ValidatedJson;
use crate::models::users_model::get_user_by_id;
use crate::models::users_model::{self, CreateUserModel};
use crate::utils::generate_hash;
use crate::{AppState, Response};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use sqlx::PgPool;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
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

    let password_hash = generate_hash(&body.password)?;
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

pub async fn create_user_internal(
    pool: &PgPool,
    username: String,
    email: String,
    password: String,
) -> Result<CreateUserResponse, AppError> {
    let email_normalizado = email.to_lowercase().trim().to_string();
    let username_normalizado = username.trim().to_string();

    if username_normalizado.is_empty() || email_normalizado.is_empty() {
        return Err(AppError::BadRequest(
            "Nome de usuário e e-mail não podem ser vazios".to_string(),
        ));
    }

    let password_hash = generate_hash(&password)?;
    let user_model = CreateUserModel {
        username: username_normalizado,
        email: email_normalizado,
        password_hash,
    };
    let user = users_model::create_user(pool, user_model)
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
    Ok(CreateUserResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
    })
}

#[derive(Serialize)]
pub struct GetUserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[instrument(skip(state))]
pub async fn get_user_me(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Response<GetUserResponse> {
    let id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("O id está inválido.".to_string()))?;
    let Some(user) = get_user_by_id(&state.pool, id).await? else {
        return Err(AppError::UserIsNotExists);
    };

    Ok((
        StatusCode::OK,
        Json(GetUserResponse {
            id: user.id.to_string(),
            email: user.email,
            username: user.username,
            created_at: user.created_at,
            updated_at: user.created_at,
        }),
    ))
}
