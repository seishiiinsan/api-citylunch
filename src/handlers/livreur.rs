use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rand::Rng;
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::{
    email::send_credentials_email,
    errors::AppError,
    models::livreur::{CreateLivreurDto, LivreurResponse, UpdateLivreurDto},
    routes::AppState,
};

fn generate_password() -> String {
    // OsRng : source cryptographiquement sûre (CSPRNG)
    rand::rngs::OsRng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16) // 16 chars → ~95 bits d'entropie (alphanumérique)
        .map(char::from)
        .collect()
}

const LIVREUR_RESPONSE_QUERY: &str =
    "SELECT id, nom, prenom, email, disponible, position_lat, position_lng, position_at, created_at, updated_at FROM livreurs";

fn row_to_livreur(row: &sqlx::postgres::PgRow) -> Result<LivreurResponse, AppError> {
    Ok(LivreurResponse {
        id: row.try_get("id").map_err(|_| AppError::InternalServerError)?,
        nom: row.try_get("nom").map_err(|_| AppError::InternalServerError)?,
        prenom: row
            .try_get("prenom")
            .map_err(|_| AppError::InternalServerError)?,
        email: row
            .try_get("email")
            .map_err(|_| AppError::InternalServerError)?,
        disponible: row
            .try_get("disponible")
            .map_err(|_| AppError::InternalServerError)?,
        position_lat: row
            .try_get("position_lat")
            .map_err(|_| AppError::InternalServerError)?,
        position_lng: row
            .try_get("position_lng")
            .map_err(|_| AppError::InternalServerError)?,
        position_at: row
            .try_get("position_at")
            .map_err(|_| AppError::InternalServerError)?,
        created_at: row
            .try_get("created_at")
            .map_err(|_| AppError::InternalServerError)?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|_| AppError::InternalServerError)?,
    })
}

pub async fn list_livreurs(
    State(state): State<AppState>,
) -> Result<Json<Vec<LivreurResponse>>, AppError> {
    let rows = sqlx::query(&format!("{} ORDER BY created_at DESC", LIVREUR_RESPONSE_QUERY))
        .fetch_all(&state.pool)
        .await?;

    let livreurs: Result<Vec<LivreurResponse>, AppError> = rows.iter().map(row_to_livreur).collect();
    Ok(Json(livreurs?))
}

pub async fn get_livreur(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LivreurResponse>, AppError> {
    let row = sqlx::query(&format!("{} WHERE id = $1", LIVREUR_RESPONSE_QUERY))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(row_to_livreur(&row)?))
}

pub async fn create_livreur(
    State(state): State<AppState>,
    Json(dto): Json<CreateLivreurDto>,
) -> Result<(StatusCode, Json<LivreurResponse>), AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let exists = sqlx::query("SELECT id FROM livreurs WHERE email = $1")
        .bind(&dto.email)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::Conflict(
            "Un livreur avec cet email existe déjà".to_string(),
        ));
    }

    let plain_password = generate_password();
    let hashed = bcrypt::hash(&plain_password, 12).map_err(|_| AppError::InternalServerError)?;

    let row = sqlx::query(
        "INSERT INTO livreurs (nom, prenom, email, mot_de_passe)
         VALUES ($1, $2, $3, $4)
         RETURNING id, nom, prenom, email, disponible, position_lat, position_lng, position_at, created_at, updated_at"
    )
    .bind(&dto.nom)
    .bind(&dto.prenom)
    .bind(&dto.email)
    .bind(&hashed)
    .fetch_one(&state.pool)
    .await?;

    let livreur = row_to_livreur(&row)?;

    // Créer automatiquement un sac vide
    sqlx::query("INSERT INTO sacs (livreur_id) VALUES ($1)")
        .bind(livreur.id)
        .execute(&state.pool)
        .await?;

    // Envoyer email avec les credentials
    send_credentials_email(&state.config, &livreur.email, &livreur.nom, &plain_password)
        .await
        .unwrap_or_else(|e| tracing::warn!("Failed to send email: {:?}", e));

    Ok((StatusCode::CREATED, Json(livreur)))
}

pub async fn update_livreur(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateLivreurDto>,
) -> Result<Json<LivreurResponse>, AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let existing_row = sqlx::query(&format!("{} WHERE id = $1", LIVREUR_RESPONSE_QUERY))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let existing = row_to_livreur(&existing_row)?;

    if let Some(ref new_email) = dto.email {
        if *new_email != existing.email {
            let conflict = sqlx::query("SELECT id FROM livreurs WHERE email = $1")
                .bind(new_email)
                .fetch_optional(&state.pool)
                .await?;
            if conflict.is_some() {
                return Err(AppError::Conflict(
                    "Un livreur avec cet email existe déjà".to_string(),
                ));
            }
        }
    }

    let nom = dto.nom.unwrap_or(existing.nom);
    let prenom = dto.prenom.unwrap_or(existing.prenom);
    let email = dto.email.unwrap_or(existing.email);
    let disponible = dto.disponible.unwrap_or(existing.disponible);
    let position_changed = dto.position_lat.is_some() || dto.position_lng.is_some();
    let position_lat = dto.position_lat.or(existing.position_lat);
    let position_lng = dto.position_lng.or(existing.position_lng);

    let row = sqlx::query(
        "UPDATE livreurs
         SET nom = $1, prenom = $2, email = $3, disponible = $4,
             position_lat = $5, position_lng = $6,
             position_at = CASE WHEN $7 THEN NOW() ELSE position_at END,
             updated_at = NOW()
         WHERE id = $8
         RETURNING id, nom, prenom, email, disponible, position_lat, position_lng, position_at, created_at, updated_at"
    )
    .bind(nom)
    .bind(prenom)
    .bind(email)
    .bind(disponible)
    .bind(position_lat)
    .bind(position_lng)
    .bind(position_changed)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(row_to_livreur(&row)?))
}

pub async fn delete_livreur(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let exists = sqlx::query("SELECT id FROM livreurs WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    sqlx::query("DELETE FROM livreurs WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
