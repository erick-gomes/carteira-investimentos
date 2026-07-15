use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request, rejection::JsonRejection},
    http::{header, request::Parts},
};
use jwt_simple::prelude::*;
use validator::Validate;

use crate::{AppState, errors::AppError};

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

pub struct AuthenticatedUser(pub String);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
        {
            if !auth_header.starts_with("Bearer ") {
                return Err(AppError::Unauthorized);
            }
            auth_header[7..].trim().to_string()
        } else if let Some(cookie_header) = parts
            .headers
            .get(header::COOKIE)
            .and_then(|header| header.to_str().ok())
        {
            cookie_header
                .split(';')
                .map(|s| s.trim())
                .find(|s| s.starts_with("access_token="))
                .map(|s| s["access_token=".len()..].to_string())
                .ok_or(AppError::Unauthorized)?
        } else {
            return Err(AppError::Unauthorized);
        };

        let key = HS256Key::from_bytes(state.jwt_secret.as_bytes());
        let claims = key
            .verify_token::<NoCustomClaims>(&token, None)
            .map_err(|_| AppError::Unauthorized)?;
        Ok(AuthenticatedUser(
            claims.subject.ok_or(AppError::Unauthorized)?,
        ))
    }
}
