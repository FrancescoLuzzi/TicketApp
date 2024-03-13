use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::Context;
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use rand;
use secrecy::{ExposeSecret, SecretString};

pub fn compute_password_hash(password: SecretString) -> Result<SecretString, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(SecretString::new(password_hash))
}

pub async fn hash_password(password: SecretString) -> Result<SecretString, anyhow::Error> {
    spawn_blocking_with_tracing(move || compute_password_hash(password))
        .await?
        .context("Failed to hash password")
}
