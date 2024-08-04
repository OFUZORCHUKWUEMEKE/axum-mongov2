use crate::error::AppError;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str) -> Result<String, AppError> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_secs(60 * 60))
        .expect("Valid timestamp")
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|_| AppError::Auth("Failed to create token".to_string()))
}

pub struct AuthUser(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| AppError::Auth("Missing authorization header".to_string()))?;
        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::Auth("Invalid authpurization header".to_string()))?;
        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::Auth("Invalid authorization header".to_string()));
        }
        let token = &auth_str["Bearer ".len()..];
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Auth("Invalid token".to_string()))?;

        Ok(AuthUser(token_data.claims.sub))
    }
}
