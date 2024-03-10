use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use rand;
use secrecy::{ExposeSecret, SecretString};

pub fn is_password_strong(password: &SecretString) -> bool {
    let password = password.expose_secret();
    if password.len() < 8 {
        return false;
    }
    let mut score = 1;

    for c in password.chars() {
        if c.is_lowercase() {
            score |= 0b0010;
            continue;
        }
        if c.is_uppercase() {
            score |= 0b0100;
            continue;
        }
        if c.is_ascii_digit() {
            score |= 0b1000;
            continue;
        }
    }
    score == 15
}

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
