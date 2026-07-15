use crate::models::refresh_token_model::{
    CreateRefreshTokenModel, create_refresh_token, delete_refresh_token, get_refresh_token_by_hash,
};
use crate::{
    AppState,
    errors::AppError,
    extractors::ValidatedJson,
    models::users_model::{get_user_by_id, get_user_by_username},
    utils::verify_password,
};
use anyhow::anyhow;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::DateTime;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::Duration as TimeDuration;
use tracing::instrument;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct AuthRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "O nome de usuário deve ter entre 3 e 50 caracteres"
    ))]
    pub username: String,
    #[validate(length(
        min = 6,
        max = 100,
        message = "A senha deve ter entre 6 e 100 caracteres"
    ))]
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
}

pub async fn authenticate_user_internal(
    pool: &PgPool,
    jar: CookieJar,
    body: AuthRequest,
    jwt_secret: &str,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AppError> {
    let user = get_user_by_username(pool, &body.username).await?;
    let Some(user) = user else {
        return Err(AppError::BadRequest(
            "Usuário ou senha inválidos".to_string(),
        ));
    };
    if !(verify_password(&body.password, &user.password_hash)?) {
        return Err(AppError::BadRequest(
            "Usuário ou senha inválidos".to_string(),
        ));
    }
    let claims_access_token =
        Claims::create(Duration::from_mins(15)).with_subject(user.id.to_string());
    let claims_refresh_token =
        Claims::create(Duration::from_days(7)).with_subject(user.id.to_string());

    let expires_at_refresh_token = claims_refresh_token
        .expires_at
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
        .ok_or(anyhow!("Não foi possível converter o tempo."))?;

    let key = HS256Key::from_bytes(jwt_secret.as_bytes());

    let access_token = key.authenticate(claims_access_token)?;
    let refresh_token = key.authenticate(claims_refresh_token)?;

    create_refresh_token(
        pool,
        CreateRefreshTokenModel {
            user_id: user.id,
            token_hash: refresh_token.clone(),
            expires_at: expires_at_refresh_token,
        },
    )
    .await?;
    let cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/api/v1/auth")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::days(7))
        .build();
    Ok((
        StatusCode::OK,
        jar.add(cookie),
        Json(AuthResponse { access_token }),
    ))
}

#[instrument]
pub async fn authenticate_user(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(body): ValidatedJson<AuthRequest>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AppError> {
    authenticate_user_internal(&state.pool, jar, body, &state.jwt_secret).await
}

#[instrument]
pub async fn refresh_access_token(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<Json<AuthResponse>, AppError>) {
    let result = async {
        let refresh_token = jar
            .get("refresh_token")
            .map(|cookie| cookie.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        let key = HS256Key::from_bytes(state.jwt_secret.as_bytes());
        let claims = key
            .verify_token::<NoCustomClaims>(&refresh_token, None)
            .map_err(|_| AppError::Unauthorized)?;

        let user_id = claims.subject.ok_or(AppError::Unauthorized)?;
        let user_id = Uuid::parse_str(&user_id).map_err(|_| AppError::Unauthorized)?;

        let stored_refresh_token = get_refresh_token_by_hash(&state.pool, &refresh_token)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if stored_refresh_token.expires_at < chrono::Utc::now() {
            let _ = delete_refresh_token(&state.pool, &refresh_token).await;
            return Err(AppError::Unauthorized);
        }

        let user = get_user_by_id(&state.pool, user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let claims_access_token =
            Claims::create(Duration::from_mins(15)).with_subject(user.id.to_string());
        let access_token = key.authenticate(claims_access_token)?;

        Ok(Json(AuthResponse { access_token }))
    }
    .await;

    match result {
        Ok(response) => (jar, Ok(response)),
        Err(AppError::Unauthorized) => {
            let clear_cookie = Cookie::build(("refresh_token", ""))
                .path("/api/v1/auth")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Strict)
                .max_age(TimeDuration::ZERO)
                .build();

            (jar.add(clear_cookie), Err(AppError::Unauthorized))
        }
        Err(other_error) => (jar, Err(other_error)),
    }
}
