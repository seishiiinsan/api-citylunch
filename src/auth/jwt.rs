use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_token(
    livreur_id: Uuid,
    secret: &str,
    expiration_hours: u64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let exp =
        (now + chrono::Duration::hours(expiration_hours as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        sub: livreur_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::InternalServerError)
}

pub fn validate_token(token: &str, secret: &str) -> Result<TokenData<Claims>, AppError> {
    let mut validation = Validation::default();
    // Exiger explicitement les claims critiques
    validation.set_required_spec_claims(&["exp", "sub", "iat"]);
    // Vérifier l'expiration (déjà vrai par défaut, rendu explicite)
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized)
}
