use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TypeProduit {
    Plat,
    Dessert,
}

impl TypeProduit {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "plat" => Some(TypeProduit::Plat),
            "dessert" => Some(TypeProduit::Dessert),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TypeProduit::Plat => "plat",
            TypeProduit::Dessert => "dessert",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ProduitRow {
    pub id: Uuid,
    pub nom: String,
    pub description: Option<String>,
    pub prix: Decimal,
    pub type_produit: String,
    pub disponible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Produit {
    pub id: Uuid,
    pub nom: String,
    pub description: Option<String>,
    pub prix: Decimal,
    pub type_produit: TypeProduit,
    pub disponible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProduitRow> for Produit {
    fn from(row: ProduitRow) -> Self {
        Produit {
            id: row.id,
            nom: row.nom,
            description: row.description,
            prix: row.prix,
            type_produit: TypeProduit::from_str(&row.type_produit)
                .unwrap_or(TypeProduit::Plat),
            disponible: row.disponible,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProduitDto {
    #[validate(length(min = 1, message = "Le nom ne peut pas être vide"))]
    pub nom: String,
    pub description: Option<String>,
    pub prix: f64,
    pub type_produit: TypeProduit,
    pub disponible: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProduitDto {
    #[validate(length(min = 1, message = "Le nom ne peut pas être vide"))]
    pub nom: Option<String>,
    pub description: Option<String>,
    pub prix: Option<f64>,
    pub type_produit: Option<TypeProduit>,
    pub disponible: Option<bool>,
}
