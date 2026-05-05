use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::middleware::AuthenticatedLivreur,
    domain,
    errors::AppError,
    models::{
        mouvement_stock::{MouvementResponse, MouvementsListResponse, TypeMouvement},
        sac::{AddProduitDto, FinServiceResponse, RetraitProduitDto, SacItemResponse, SacResponse},
    },
    routes::AppState,
};

/// Adaptateur : convertit le résultat domain (String) en AppError pour Axum.
fn valider_retrait(quantite_sac: i32, quantite_retrait: i32) -> Result<(), AppError> {
    domain::valider_retrait(quantite_sac, quantite_retrait)
        .map_err(AppError::BadRequest)
}

#[utoipa::path(
    get,
    path = "/api/v1/sac",
    tag = "sac",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Contenu du sac du livreur authentifié", body = SacResponse),
        (status = 401, description = "Non authentifié", body = ErrorResponse),
        (status = 404, description = "Sac introuvable", body = ErrorResponse),
    )
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/sac/produits",
    tag = "sac",
    security(("bearerAuth" = [])),
    request_body = AddProduitDto,
    responses(
        (status = 201, description = "Produit ajouté au sac (quantité incrémentée si déjà présent)"),
        (status = 400, description = "Produit indisponible ou quantité invalide", body = ErrorResponse),
        (status = 401, description = "Non authentifié", body = ErrorResponse),
        (status = 404, description = "Produit introuvable", body = ErrorResponse),
    )
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/sac/produits/{produit_id}",
    tag = "sac",
    security(("bearerAuth" = [])),
    params(("produit_id" = Uuid, Path, description = "Identifiant du produit à retirer")),
    request_body = RetraitProduitDto,
    responses(
        (status = 204, description = "Produit retiré du sac"),
        (status = 400, description = "Quantité de retrait supérieure au stock", body = ErrorResponse),
        (status = 401, description = "Non authentifié", body = ErrorResponse),
        (status = 404, description = "Produit non trouvé dans le sac", body = ErrorResponse),
    )
)]
pub async fn remove_produit_sac(
    State(state): State<AppState>,
    auth: AuthenticatedLivreur,
    Path(produit_id): Path<Uuid>,
    Json(dto): Json<RetraitProduitDto>,
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

    let item_row = sqlx::query(
        "SELECT quantite FROM sac_items WHERE sac_id = $1 AND produit_id = $2",
    )
    .bind(sac_id)
    .bind(produit_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let quantite_sac: i32 = item_row
        .try_get("quantite")
        .map_err(|_| AppError::InternalServerError)?;

    valider_retrait(quantite_sac, dto.quantite)?;

    let nouvelle_quantite = quantite_sac - dto.quantite;
    if nouvelle_quantite == 0 {
        sqlx::query("DELETE FROM sac_items WHERE sac_id = $1 AND produit_id = $2")
            .bind(sac_id)
            .bind(produit_id)
            .execute(&state.pool)
            .await?;
    } else {
        sqlx::query(
            "UPDATE sac_items SET quantite = $1 WHERE sac_id = $2 AND produit_id = $3",
        )
        .bind(nouvelle_quantite)
        .bind(sac_id)
        .bind(produit_id)
        .execute(&state.pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO mouvements_stock (livreur_id, produit_id, quantite, type_mouvement) VALUES ($1, $2, $3, $4::type_mouvement)"
    )
    .bind(auth.livreur_id)
    .bind(produit_id)
    .bind(dto.quantite)
    .bind(TypeMouvement::Retrait.as_str())
    .execute(&state.pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/livreurs/{id}/sac",
    tag = "livreurs",
    params(("id" = Uuid, Path, description = "Identifiant du livreur")),
    responses(
        (status = 200, description = "Sac du livreur (vue gérant)", body = SacResponse),
        (status = 404, description = "Livreur introuvable", body = ErrorResponse),
    )
)]
/// GET /livreurs/:id/sac — consultation du sac d'un livreur (accessible sans JWT, pour le gérant)
pub async fn get_sac_by_livreur(
    State(state): State<AppState>,
    Path(livreur_id): Path<Uuid>,
) -> Result<Json<SacResponse>, AppError> {
    // Vérifier que le livreur existe
    let livreur_exists = sqlx::query("SELECT id FROM livreurs WHERE id = $1")
        .bind(livreur_id)
        .fetch_optional(&state.pool)
        .await?;
    if livreur_exists.is_none() {
        return Err(AppError::NotFound);
    }

    let sac_row = sqlx::query("SELECT id, livreur_id FROM sacs WHERE livreur_id = $1")
        .bind(livreur_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let sac_id: Uuid = sac_row.try_get("id").map_err(|_| AppError::InternalServerError)?;
    let livreur_id: Uuid = sac_row.try_get("livreur_id").map_err(|_| AppError::InternalServerError)?;

    let item_rows = sqlx::query(
        "SELECT si.produit_id, p.nom, p.type_produit::text as type_produit, si.quantite, p.prix as prix_unitaire
         FROM sac_items si
         JOIN produits p ON p.id = si.produit_id
         WHERE si.sac_id = $1",
    )
    .bind(sac_id)
    .fetch_all(&state.pool)
    .await?;

    let mut items: Vec<SacItemResponse> = Vec::new();
    let mut total_items: i64 = 0;
    for row in &item_rows {
        let quantite: i32 = row.try_get("quantite").map_err(|_| AppError::InternalServerError)?;
        total_items += quantite as i64;
        items.push(SacItemResponse {
            produit_id: row.try_get("produit_id").map_err(|_| AppError::InternalServerError)?,
            nom: row.try_get("nom").map_err(|_| AppError::InternalServerError)?,
            type_produit: row.try_get("type_produit").map_err(|_| AppError::InternalServerError)?,
            quantite,
            prix_unitaire: row.try_get("prix_unitaire").map_err(|_| AppError::InternalServerError)?,
        });
    }

    Ok(Json(SacResponse { sac_id, livreur_id, items, total_items }))
}

#[utoipa::path(
    post,
    path = "/api/v1/sac/fin-service",
    tag = "sac",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Fin de service enregistrée — sac vidé", body = FinServiceResponse),
        (status = 401, description = "Non authentifié", body = ErrorResponse),
        (status = 404, description = "Sac introuvable", body = ErrorResponse),
    )
)]
/// POST /sac/fin-service — retour des produits non livrés en fin de service
pub async fn fin_service(
    State(state): State<AppState>,
    auth: AuthenticatedLivreur,
) -> Result<Json<FinServiceResponse>, AppError> {
    let sac_row = sqlx::query("SELECT id FROM sacs WHERE livreur_id = $1")
        .bind(auth.livreur_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let sac_id: Uuid = sac_row.try_get("id").map_err(|_| AppError::InternalServerError)?;

    let items = sqlx::query("SELECT produit_id, quantite FROM sac_items WHERE sac_id = $1")
        .bind(sac_id)
        .fetch_all(&state.pool)
        .await?;

    let mut retours = 0i32;
    for row in &items {
        let produit_id: Uuid = row.try_get("produit_id").map_err(|_| AppError::InternalServerError)?;
        let quantite: i32 = row.try_get("quantite").map_err(|_| AppError::InternalServerError)?;

        sqlx::query(
            "INSERT INTO mouvements_stock (livreur_id, produit_id, quantite, type_mouvement)
             VALUES ($1, $2, $3, $4::type_mouvement)",
        )
        .bind(auth.livreur_id)
        .bind(produit_id)
        .bind(quantite)
        .bind("retour_fin_service")
        .execute(&state.pool)
        .await?;

        retours += quantite;
    }

    // Vider le sac
    sqlx::query("DELETE FROM sac_items WHERE sac_id = $1")
        .bind(sac_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(FinServiceResponse {
        message: "Fin de service enregistrée".to_string(),
        produits_retournes: retours,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/livreurs/{id}/mouvements",
    tag = "livreurs",
    params(("id" = Uuid, Path, description = "Identifiant du livreur")),
    responses(
        (status = 200, description = "Historique des mouvements de stock", body = MouvementsListResponse),
        (status = 404, description = "Livreur introuvable", body = ErrorResponse),
    )
)]
/// GET /livreurs/:id/mouvements — historique des mouvements d'un livreur
pub async fn get_mouvements(
    State(state): State<AppState>,
    Path(livreur_id): Path<Uuid>,
) -> Result<Json<MouvementsListResponse>, AppError> {
    let exists = sqlx::query("SELECT id FROM livreurs WHERE id = $1")
        .bind(livreur_id)
        .fetch_optional(&state.pool)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    let rows = sqlx::query(
        "SELECT ms.id, ms.produit_id, p.nom as produit_nom,
                ms.quantite, ms.type_mouvement::text as type_mouvement, ms.created_at
         FROM mouvements_stock ms
         JOIN produits p ON p.id = ms.produit_id
         WHERE ms.livreur_id = $1
         ORDER BY ms.created_at DESC",
    )
    .bind(livreur_id)
    .fetch_all(&state.pool)
    .await?;

    let mouvements: Result<Vec<MouvementResponse>, AppError> = rows
        .iter()
        .map(|row| -> Result<MouvementResponse, AppError> {
            Ok(MouvementResponse {
                id: row.try_get("id").map_err(|_| AppError::InternalServerError)?,
                produit_id: row.try_get("produit_id").map_err(|_| AppError::InternalServerError)?,
                produit_nom: row.try_get("produit_nom").map_err(|_| AppError::InternalServerError)?,
                quantite: row.try_get("quantite").map_err(|_| AppError::InternalServerError)?,
                type_mouvement: row.try_get("type_mouvement").map_err(|_| AppError::InternalServerError)?,
                created_at: row.try_get("created_at").map_err(|_| AppError::InternalServerError)?,
            })
        })
        .collect();

    Ok(Json(MouvementsListResponse { mouvements: mouvements? }))
}

// Les tests unitaires sont dans tests/business_rules.rs
