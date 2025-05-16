use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, errors::Error as JwtError};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hashed)
}

pub fn create_jwt_for(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 30))
        .expect("valid timestamp")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards 🙃")
        .as_secs() as usize;

    let claims = Claims {
        username: username.to_string(),
        exp: expiration,
    };

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes())
    )
}


pub fn verify_jwt(token: &str) -> Result<Claims, JwtError> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default()
    )?;

    Ok(token_data.claims)
}


