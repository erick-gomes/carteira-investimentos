use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct CreateAssetModel {
    pub user_id: Uuid,
    pub name: String,
    pub ticker: Option<String>,
    pub category: String,
    pub quantity_raw: i64,
    pub average_price_cents: i64,
    pub last_acquisition_date: Option<NaiveDate>,
    pub currency: String,
}

#[derive(Debug)]
pub struct AssetModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticker: Option<String>,
    pub name: String,
    pub category: String,
    pub quantity_raw: i64,
    pub average_price_cents: i64,
    pub currency: String,
    pub last_acquisition_date: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub async fn create_asset(
    pool: &PgPool,
    asset: CreateAssetModel,
) -> Result<AssetModel, sqlx::Error> {
    let id = Uuid::now_v7();

    let created_asset = sqlx::query_as!(
        AssetModel,
        r#"
        INSERT INTO assets (
            id, user_id, ticker, name, category, 
            quantity_raw, average_price_cents, currency, 
            last_acquisition_date
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, ticker, name, category, 
                  quantity_raw, average_price_cents, currency, 
                  last_acquisition_date, created_at, updated_at
        "#,
        id,
        asset.user_id,
        asset.ticker,
        asset.name,
        asset.category,
        asset.quantity_raw,
        asset.average_price_cents,
        asset.currency,
        asset.last_acquisition_date
    )
    .fetch_one(pool)
    .await?;

    Ok(created_asset)
}

pub async fn get_assets_by_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<AssetModel>, sqlx::Error> {
    let assets = sqlx::query_as!(
        AssetModel,
        r#"
        SELECT id, user_id, ticker, name, category, quantity_raw,
               average_price_cents, currency, last_acquisition_date,
               created_at, updated_at
        FROM assets
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(assets)
}

pub async fn patch_asset(
    pool: &PgPool,
    asset_id: Uuid,
    user_id: Uuid,
    name: Option<String>,
    category: Option<String>,
    quantity_raw: Option<i64>,
    average_price_cents: Option<i64>,
    last_acquisition_date: Option<NaiveDate>,
) -> Result<Option<AssetModel>, sqlx::Error> {
    let updated_asset = sqlx::query_as!(
        AssetModel,
        r#"
        UPDATE assets
        SET name = COALESCE($1, name),
            category = COALESCE($2, category),
            quantity_raw = COALESCE($3, quantity_raw),
            average_price_cents = COALESCE($4, average_price_cents),
            last_acquisition_date = COALESCE($5, last_acquisition_date),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $6 AND user_id = $7
        RETURNING id, user_id, ticker, name, category, quantity_raw,
                  average_price_cents, currency, last_acquisition_date,
                  created_at, updated_at
        "#,
        name,
        category,
        quantity_raw,
        average_price_cents,
        last_acquisition_date,
        asset_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(updated_asset)
}

pub async fn delete_asset(
    pool: &PgPool,
    asset_id: Uuid,
    user_id: Uuid,
) -> Result<Option<AssetModel>, sqlx::Error> {
    let deleted_asset = sqlx::query_as!(
        AssetModel,
        r#"
        DELETE FROM assets
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, ticker, name, category, quantity_raw,
                  average_price_cents, currency, last_acquisition_date,
                  created_at, updated_at
        "#,
        asset_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(deleted_asset)
}
