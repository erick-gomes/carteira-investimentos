use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn generate_hash(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| {
            tracing::error!("Erro ao gerar hash da senha: {:?}", error);
            anyhow!("Erro ao gerar hash da senha")
        })?
        .to_string();
    Ok(password_hash)
}
