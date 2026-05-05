use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rust_decimal::Decimal;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::AppError,
    models::produit::{CreateProduitDto, Produit, ProduitRow, UpdateProduitDto},
    routes::AppState,
};

fn row_to_produit(row: &sqlx::postgres::PgRow) -> Result<Produit, AppError> {
    let row = ProduitRow {
        id: row.try_get("id").map_err(|_| AppError::InternalServerError)?,
        nom: row.try_get("nom").map_err(|_| AppError::InternalServerError)?,
        description: row
            .try_get("description")
            .map_err(|_| AppError::InternalServerError)?,
        prix: row.try_get("prix").map_err(|_| AppError::InternalServerError)?,
        type_produit: row
            .try_get("type_produit")
            .map_err(|_| AppError::InternalServerError)?,
        disponible: row
            .try_get("disponible")
            .map_err(|_| AppError::InternalServerError)?,
        created_at: row
            .try_get("created_at")
            .map_err(|_| AppError::InternalServerError)?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|_| AppError::InternalServerError)?,
    };
    Ok(row.into())
}

pub async fn list_produits(
    State(state): State<AppState>,
) -> Result<Json<Vec<Produit>>, AppError> {
    let rows = sqlx::query(
        "SELECT id, nom, description, prix, type_produit::text as type_produit, disponible, created_at, updated_at FROM produits ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await?;

    let produits: Result<Vec<Produit>, AppError> = rows.iter().map(row_to_produit).collect();
    Ok(Json(produits?))
}

pub async fn get_produit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Produit>, AppError> {
    let row = sqlx::query(
        "SELECT id, nom, description, prix, type_produit::text as type_produit, disponible, created_at, updated_at FROM produits WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(row_to_produit(&row)?))
}

pub async fn create_produit(
    State(state): State<AppState>,
    Json(dto): Json<CreateProduitDto>,
) -> Result<(StatusCode, Json<Produit>), AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    if dto.prix < 0.0 {
        return Err(AppError::BadRequest("Le prix doit être >= 0".to_string()));
    }

    let prix = Decimal::from_str(&dto.prix.to_string())
        .map_err(|_| AppError::BadRequest("Prix invalide".to_string()))?;

    let row = sqlx::query(
        "INSERT INTO produits (nom, description, prix, type_produit, disponible)
         VALUES ($1, $2, $3, $4::type_produit, $5)
         RETURNING id, nom, description, prix, type_produit::text as type_produit, disponible, created_at, updated_at"
    )
    .bind(&dto.nom)
    .bind(&dto.description)
    .bind(prix)
    .bind(dto.type_produit.as_str())
    .bind(dto.disponible.unwrap_or(true))
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row_to_produit(&row)?)))
}

pub async fn update_produit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateProduitDto>,
) -> Result<Json<Produit>, AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    if let Some(prix) = dto.prix {
        if prix < 0.0 {
            return Err(AppError::BadRequest("Le prix doit être >= 0".to_string()));
        }
    }

    let existing_row = sqlx::query(
        "SELECT id, nom, description, prix, type_produit::text as type_produit, disponible, created_at, updated_at FROM produits WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let existing = row_to_produit(&existing_row)?;

    let nom = dto.nom.unwrap_or(existing.nom);
    let description = dto.description.or(existing.description);
    let prix = match dto.prix {
        Some(p) => Decimal::from_str(&p.to_string())
            .map_err(|_| AppError::BadRequest("Prix invalide".to_string()))?,
        None => existing.prix,
    };
    let type_produit = dto.type_produit.unwrap_or(existing.type_produit);
    let disponible = dto.disponible.unwrap_or(existing.disponible);

    let row = sqlx::query(
        "UPDATE produits SET nom = $1, description = $2, prix = $3, type_produit = $4::type_produit, disponible = $5, updated_at = NOW()
         WHERE id = $6
         RETURNING id, nom, description, prix, type_produit::text as type_produit, disponible, created_at, updated_at"
    )
    .bind(nom)
    .bind(description)
    .bind(prix)
    .bind(type_produit.as_str())
    .bind(disponible)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(row_to_produit(&row)?))
}

pub async fn delete_produit(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let exists = sqlx::query("SELECT id FROM produits WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    let in_sac = sqlx::query("SELECT id FROM sac_items WHERE produit_id = $1 LIMIT 1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?;

    if in_sac.is_some() {
        return Err(AppError::Conflict(
            "Le produit est présent dans un sac actif et ne peut pas être supprimé".to_string(),
        ));
    }

    sqlx::query("DELETE FROM produits WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
