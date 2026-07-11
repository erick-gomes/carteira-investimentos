use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct RefreshTokenModel {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct CreateRefreshTokenModel {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn create_refresh_token(
    pool: &sqlx::PgPool,
    refresh_token_model: CreateRefreshTokenModel,
) -> Result<RefreshTokenModel, sqlx::Error> {
    let refresh_token = sqlx::query_as!(
        RefreshTokenModel,
        r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING user_id, token_hash, expires_at, created_at
            "#,
        refresh_token_model.user_id,
        refresh_token_model.token_hash,
        refresh_token_model.expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(refresh_token)
}

pub async fn get_refresh_token_by_hash(
    pool: &sqlx::PgPool,
    token_hash: &str,
) -> Result<Option<RefreshTokenModel>, sqlx::Error> {
    let refresh_token = sqlx::query_as!(
        RefreshTokenModel,
        r#"
            SELECT user_id, token_hash, expires_at, created_at
            FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        token_hash
    )
    .fetch_optional(pool)
    .await?;

    Ok(refresh_token)
}

pub async fn delete_refresh_token(
    pool: &sqlx::PgPool,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM refresh_tokens
            WHERE token_hash = $1
            "#,
        token_hash
    )
    .execute(pool)
    .await?;

    Ok(())
}
