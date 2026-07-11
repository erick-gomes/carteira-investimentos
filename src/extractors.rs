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
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized);
        }
        let token = auth_header[7..].trim();
        let key = HS256Key::from_bytes(state.jwt_secret.as_bytes());
        let claims = key
            .verify_token::<NoCustomClaims>(token, None)
            .map_err(|_| AppError::Unauthorized)?;
        Ok(AuthenticatedUser(
            claims.subject.ok_or(AppError::Unauthorized)?,
        ))
    }
}
