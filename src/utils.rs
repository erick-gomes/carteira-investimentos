use anyhow::anyhow;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
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

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, anyhow::Error> {
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &PasswordHash::new(&password_hash)?) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(error) => {
            tracing::error!("Erro ao verificar senha: {:?}", error);
            Err(anyhow!("Erro ao verificar senha"))
        }
    }
}
