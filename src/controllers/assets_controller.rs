use crate::AppState;
use crate::Response;
use crate::errors::AppError;
use crate::extractors::{AuthenticatedUser, ValidatedJson};
use crate::models::assets_model::{self, CreateAssetModel};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAssetRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "O nome do ativo deve ter entre 1 e 100 caracteres"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 50,
        message = "A categoria deve ter entre 1 e 50 caracteres"
    ))]
    pub category: String,

    #[validate(length(
        equal = 3,
        message = "A moeda deve ser um código de exatamente 3 caracteres (ex: BRL, USD)"
    ))]
    pub currency: String,

    #[validate(length(
        min = 1,
        max = 20,
        message = "O ticker deve ter entre 1 e 20 caracteres"
    ))]
    pub ticker: Option<String>,

    #[validate(range(
        min = 1,
        message = "A quantidade deve ser maior que zero"
    ))]
    pub quantity_raw: i64,

    #[validate(range(
        min = 0,
        message = "O preço médio não pode ser negativo"
    ))]
    pub average_price_cents: i64,

    pub last_acquisition_date: NaiveDate,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PatchAssetRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "O nome do ativo deve ter entre 1 e 100 caracteres"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 1,
        max = 50,
        message = "A categoria deve ter entre 1 e 50 caracteres"
    ))]
    pub category: Option<String>,

    #[validate(range(
        min = 1,
        message = "A quantidade deve ser maior que zero"
    ))]
    pub quantity_raw: Option<i64>,

    #[validate(range(
        min = 0,
        message = "O preço médio não pode ser negativo"
    ))]
    pub average_price_cents: Option<i64>,

    pub last_acquisition_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub ticker: Option<String>,
    pub name: String,
    pub category: String,
    pub quantity: f64,
    pub average_price: f64,
    pub total_invested: f64,
    pub currency: String,
    pub last_acquisition_date: Option<NaiveDate>,
}

pub type CreateAssetResponse = AssetResponse;
pub type PatchAssetResponse = AssetResponse;
pub type DeleteAssetResponse = AssetResponse;

impl AssetResponse {
    fn from_model(asset: assets_model::AssetModel) -> Self {
        let quantity = asset.quantity_raw as f64 / 100.0;
        let average_price = asset.average_price_cents as f64 / 100.0;

        Self {
            id: asset.id,
            ticker: asset.ticker,
            name: asset.name,
            category: asset.category,
            quantity,
            average_price,
            total_invested: quantity * average_price,
            currency: asset.currency,
            last_acquisition_date: asset.last_acquisition_date,
        }
    }
}

pub async fn get_all_assets(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Response<Vec<AssetResponse>> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("O id do usuário está inválido.".to_string()))?;

    let assets = assets_model::get_assets_by_user(&state.pool, user_id).await?;
    let assets = assets.into_iter().map(AssetResponse::from_model).collect();

    Ok((StatusCode::OK, Json(assets)))
}

pub async fn create_asset(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    ValidatedJson(body): ValidatedJson<CreateAssetRequest>,
) -> Response<CreateAssetResponse> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("O id do usuário está inválido.".to_string()))?;

    let create_model = CreateAssetModel {
        user_id,
        name: body.name,
        ticker: body.ticker,
        category: body.category,
        quantity_raw: body.quantity_raw,
        average_price_cents: body.average_price_cents,
        last_acquisition_date: Some(body.last_acquisition_date),
        currency: body.currency,
    };

    let created_asset = assets_model::create_asset(&state.pool, create_model).await?;
    let response = AssetResponse::from_model(created_asset);

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn patch_asset(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(asset_id): Path<String>,
    ValidatedJson(body): ValidatedJson<PatchAssetRequest>,
) -> Response<PatchAssetResponse> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("O id do usuário está inválido.".to_string()))?;
    let asset_id = Uuid::parse_str(&asset_id)
        .map_err(|_| AppError::BadRequest("O id do ativo está inválido.".to_string()))?;

    let updated_asset = assets_model::patch_asset(
        &state.pool,
        asset_id,
        user_id,
        body.name,
        body.category,
        body.quantity_raw,
        body.average_price_cents,
        body.last_acquisition_date,
    )
    .await?
    .ok_or_else(|| AppError::BadRequest("Ativo não encontrado.".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(AssetResponse::from_model(updated_asset)),
    ))
}

pub async fn delete_asset(
    State(state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(asset_id): Path<String>,
) -> Response<DeleteAssetResponse> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("O id do usuário está inválido.".to_string()))?;
    let asset_id = Uuid::parse_str(&asset_id)
        .map_err(|_| AppError::BadRequest("O id do ativo está inválido.".to_string()))?;

    let deleted_asset = assets_model::delete_asset(&state.pool, asset_id, user_id)
        .await?
        .ok_or_else(|| AppError::BadRequest("Ativo não encontrado.".to_string()))?;

    Ok((
        StatusCode::OK,
        Json(AssetResponse::from_model(deleted_asset)),
    ))
}
