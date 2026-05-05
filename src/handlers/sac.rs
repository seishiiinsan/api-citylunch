use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rust_decimal::Decimal;
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::middleware::AuthenticatedLivreur,
    errors::AppError,
    models::{
        mouvement_stock::TypeMouvement,
        sac::{AddProduitDto, SacItemResponse, SacResponse},
    },
    routes::AppState,
};

pub fn valider_retrait(quantite_sac: i32, quantite_retrait: i32) -> Result<(), AppError> {
    if quantite_retrait > quantite_sac {
        Err(AppError::BadRequest(format!(
            "Impossible de retirer {} unité(s) : seulement {} disponible(s) dans le sac",
            quantite_retrait, quantite_sac
        )))
    } else {
        Ok(())
    }
}

pub async fn get_sac(
    State(state): State<AppState>,
    auth: AuthenticatedLivreur,
) -> Result<Json<SacResponse>, AppError> {
    let sac_row = sqlx::query("SELECT id, livreur_id FROM sacs WHERE livreur_id = $1")
        .bind(auth.livreur_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let sac_id: Uuid = sac_row
        .try_get("id")
        .map_err(|_| AppError::InternalServerError)?;
    let livreur_id: Uuid = sac_row
        .try_get("livreur_id")
        .map_err(|_| AppError::InternalServerError)?;

    let item_rows = sqlx::query(
        "SELECT si.produit_id, p.nom, p.type_produit::text as type_produit, si.quantite, p.prix as prix_unitaire
         FROM sac_items si
         JOIN produits p ON p.id = si.produit_id
         WHERE si.sac_id = $1"
    )
    .bind(sac_id)
    .fetch_all(&state.pool)
    .await?;

    let mut items: Vec<SacItemResponse> = Vec::new();
    let mut total_items: i64 = 0;

    for row in &item_rows {
        let quantite: i32 = row
            .try_get("quantite")
            .map_err(|_| AppError::InternalServerError)?;
        total_items += quantite as i64;
        items.push(SacItemResponse {
            produit_id: row
                .try_get("produit_id")
                .map_err(|_| AppError::InternalServerError)?,
            nom: row.try_get("nom").map_err(|_| AppError::InternalServerError)?,
            type_produit: row
                .try_get("type_produit")
                .map_err(|_| AppError::InternalServerError)?,
            quantite,
            prix_unitaire: row
                .try_get("prix_unitaire")
                .map_err(|_| AppError::InternalServerError)?,
        });
    }

    Ok(Json(SacResponse {
        sac_id,
        livreur_id,
        items,
        total_items,
    }))
}

pub async fn add_produit_sac(
    State(state): State<AppState>,
    auth: AuthenticatedLivreur,
    Json(dto): Json<AddProduitDto>,
) -> Result<StatusCode, AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let sac_row = sqlx::query("SELECT id FROM sacs WHERE livreur_id = $1")
        .bind(auth.livreur_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let sac_id: Uuid = sac_row
        .try_get("id")
        .map_err(|_| AppError::InternalServerError)?;

    let produit_row = sqlx::query("SELECT id, disponible FROM produits WHERE id = $1")
        .bind(dto.produit_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let disponible: bool = produit_row
        .try_get("disponible")
        .map_err(|_| AppError::InternalServerError)?;

    if !disponible {
        return Err(AppError::BadRequest(
            "Le produit n'est pas disponible".to_string(),
        ));
    }

    sqlx::query(
        "INSERT INTO sac_items (sac_id, produit_id, quantite)
         VALUES ($1, $2, $3)
         ON CONFLICT (sac_id, produit_id) DO UPDATE SET quantite = sac_items.quantite + $3"
    )
    .bind(sac_id)
    .bind(dto.produit_id)
    .bind(dto.quantite)
    .execute(&state.pool)
    .await?;

    sqlx::query(
        "INSERT INTO mouvements_stock (livreur_id, produit_id, quantite, type_mouvement) VALUES ($1, $2, $3, $4::type_mouvement)"
    )
    .bind(auth.livreur_id)
    .bind(dto.produit_id)
    .bind(dto.quantite)
    .bind(TypeMouvement::Chargement.as_str())
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::CREATED)
}

pub async fn remove_produit_sac(
    State(state): State<AppState>,
    auth: AuthenticatedLivreur,
    Path(produit_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let sac_row = sqlx::query("SELECT id FROM sacs WHERE livreur_id = $1")
        .bind(auth.livreur_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let sac_id: Uuid = sac_row
        .try_get("id")
        .map_err(|_| AppError::InternalServerError)?;

    let item_row = sqlx::query(
        "SELECT quantite FROM sac_items WHERE sac_id = $1 AND produit_id = $2"
    )
    .bind(sac_id)
    .bind(produit_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let quantite: i32 = item_row
        .try_get("quantite")
        .map_err(|_| AppError::InternalServerError)?;

    valider_retrait(quantite, quantite)?;

    sqlx::query("DELETE FROM sac_items WHERE sac_id = $1 AND produit_id = $2")
        .bind(sac_id)
        .bind(produit_id)
        .execute(&state.pool)
        .await?;

    sqlx::query(
        "INSERT INTO mouvements_stock (livreur_id, produit_id, quantite, type_mouvement) VALUES ($1, $2, $3, $4::type_mouvement)"
    )
    .bind(auth.livreur_id)
    .bind(produit_id)
    .bind(quantite)
    .bind(TypeMouvement::Retrait.as_str())
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrait_superieur_au_stock_est_invalide() {
        let result = valider_retrait(2, 5);
        assert!(result.is_err());
    }

    #[test]
    fn retrait_egal_au_stock_est_valide() {
        let result = valider_retrait(3, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn retrait_inferieur_au_stock_est_valide() {
        let result = valider_retrait(5, 2);
        assert!(result.is_ok());
    }
}
