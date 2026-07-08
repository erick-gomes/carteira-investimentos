use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
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
            RETURNING id, username, email, password_hash
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
