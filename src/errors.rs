use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Erro de validação: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Erro na requisição JSON: {0}")]
    JsonRejection(#[from] JsonRejection),

    #[error("Erro no banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Erro interno do sistema")]
    InternalServer(#[from] anyhow::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse<T: Serialize> {
    pub errors: T,
}
impl<T: Serialize> ErrorResponse<T> {
    pub fn new(errors: T) -> Self {
        Self { errors }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Validation(errors) => {
                (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(errors))).into_response()
            }
            AppError::JsonRejection(error) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new([error.to_string()])),
            )
                .into_response(),
            AppError::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new([error.to_string()])),
            )
                .into_response(),
            AppError::InternalServer(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new([error.to_string()])),
            )
                .into_response(),
        }
    }
}
