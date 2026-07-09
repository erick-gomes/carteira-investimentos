use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateUserModel {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}
pub async fn create_user(pool: &PgPool, user: CreateUserModel) -> Result<UserModel, sqlx::Error> {
    let id = Uuid::now_v7();
    let user = sqlx::query_as!(
        UserModel,
        r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password_hash, created_at, updated_at
            "#,
        id,
        user.username,
        user.email,
        user.password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}
