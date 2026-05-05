use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::{auth::jwt::validate_token, errors::AppError, routes::AppState};

#[derive(Debug, Clone)]
pub struct AuthenticatedLivreur {
    pub livreur_id: Uuid,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedLivreur {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let token_data = validate_token(token, &state.config.jwt_secret)?;

        let livreur_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthenticatedLivreur { livreur_id })
    }
}
