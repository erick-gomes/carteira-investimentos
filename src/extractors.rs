use axum::{
    Json, extract::{FromRequest, Request, rejection::JsonRejection},
};
use validator::Validate;

use crate::errors::AppError;

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
