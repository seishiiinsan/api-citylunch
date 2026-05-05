use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sac {
    pub id: Uuid,
    pub livreur_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SacItemResponse {
    pub produit_id: Uuid,
    pub nom: String,
    pub type_produit: String,
    pub quantite: i32,
    pub prix_unitaire: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SacResponse {
    pub sac_id: Uuid,
    pub livreur_id: Uuid,
    pub items: Vec<SacItemResponse>,
    pub total_items: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddProduitDto {
    pub produit_id: Uuid,
    #[validate(range(min = 1))]
    pub quantite: i32,
}
