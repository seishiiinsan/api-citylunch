use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Sac {
    pub id: Uuid,
    pub livreur_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SacItemResponse {
    pub produit_id: Uuid,
    pub nom: String,
    pub type_produit: String,
    pub quantite: i32,
    #[schema(value_type = f64, example = 12.5)]
    pub prix_unitaire: Decimal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SacResponse {
    pub sac_id: Uuid,
    pub livreur_id: Uuid,
    pub items: Vec<SacItemResponse>,
    pub total_items: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AddProduitDto {
    pub produit_id: Uuid,
    #[validate(range(min = 1))]
    #[schema(example = 3)]
    pub quantite: i32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RetraitProduitDto {
    #[validate(range(min = 1))]
    #[schema(example = 2)]
    pub quantite: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FinServiceResponse {
    #[schema(example = "Fin de service enregistrée")]
    pub message: String,
    #[schema(example = 5)]
    pub produits_retournes: i32,
}
