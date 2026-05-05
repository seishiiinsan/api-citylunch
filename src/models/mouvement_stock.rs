use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TypeMouvement {
    Chargement,
    Retrait,
    RetourFinService,
}

impl TypeMouvement {
    pub fn as_str(&self) -> &str {
        match self {
            TypeMouvement::Chargement => "chargement",
            TypeMouvement::Retrait => "retrait",
            TypeMouvement::RetourFinService => "retour_fin_service",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct MouvementStock {
    pub id: Uuid,
    pub livreur_id: Uuid,
    pub produit_id: Uuid,
    pub quantite: i32,
    pub type_mouvement: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MouvementResponse {
    pub id: Uuid,
    pub produit_id: Uuid,
    pub produit_nom: String,
    pub quantite: i32,
    pub type_mouvement: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MouvementsListResponse {
    pub mouvements: Vec<MouvementResponse>,
}
