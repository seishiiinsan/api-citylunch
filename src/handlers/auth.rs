use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{auth::jwt::generate_token, errors::AppError, routes::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub mot_de_passe: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<LoginResponse>, AppError> {
    let row = sqlx::query("SELECT id, mot_de_passe FROM livreurs WHERE email = $1")
        .bind(&dto.email)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let id: uuid::Uuid = row.try_get("id").map_err(|_| AppError::InternalServerError)?;
    let mot_de_passe: String = row
        .try_get("mot_de_passe")
        .map_err(|_| AppError::InternalServerError)?;

    let valid =
        bcrypt::verify(&dto.mot_de_passe, &mot_de_passe).map_err(|_| AppError::InternalServerError)?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = generate_token(id, &state.config.jwt_secret, state.config.jwt_expiration_hours)?;

    Ok(Json(LoginResponse { token }))
}
