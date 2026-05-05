use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;

use crate::{auth::jwt::generate_token, errors::AppError, routes::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginDto {
    #[schema(example = "jean.dupont@citylunch.fr")]
    pub email: String,
    #[schema(example = "motDePasseRecu")]
    pub mot_de_passe: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    #[schema(example = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...")]
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Connexion réussie — retourne le token JWT", body = LoginResponse),
        (status = 401, description = "Email ou mot de passe incorrect", body = ErrorResponse),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<LoginResponse>, AppError> {
    let row = sqlx::query("SELECT id, mot_de_passe FROM livreurs WHERE email = $1")
        .bind(&dto.email)
        .fetch_optional(&state.pool)
        .await?;

    // Même réponse (401) que l'email soit inconnu ou le mot de passe faux
    // → empêche l'énumération de comptes. Le log interne distingue les deux cas.
    let row = match row {
        Some(r) => r,
        None => {
            tracing::warn!(email = %dto.email, "tentative de connexion — email inconnu");
            return Err(AppError::Unauthorized);
        }
    };

    let id: uuid::Uuid = row.try_get("id").map_err(|_| AppError::InternalServerError)?;
    let mot_de_passe: String = row
        .try_get("mot_de_passe")
        .map_err(|_| AppError::InternalServerError)?;

    let valid = bcrypt::verify(&dto.mot_de_passe, &mot_de_passe)
        .map_err(|e| {
            tracing::error!("bcrypt verify error: {:?}", e);
            AppError::InternalServerError
        })?;

    if !valid {
        tracing::warn!(email = %dto.email, livreur_id = %id, "tentative de connexion — mot de passe incorrect");
        return Err(AppError::Unauthorized);
    }

    tracing::info!(email = %dto.email, livreur_id = %id, "connexion réussie");
    let token = generate_token(id, &state.config.jwt_secret, state.config.jwt_expiration_hours)?;

    Ok(Json(LoginResponse { token }))
}
