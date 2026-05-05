use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
#[allow(dead_code)]
pub struct Livreur {
    pub id: Uuid,
    pub nom: String,
    pub prenom: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub mot_de_passe: String,
    pub disponible: bool,
    pub position_lat: Option<f64>,
    pub position_lng: Option<f64>,
    pub position_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct LivreurResponse {
    pub id: Uuid,
    pub nom: String,
    pub prenom: String,
    pub email: String,
    pub disponible: bool,
    pub position_lat: Option<f64>,
    pub position_lng: Option<f64>,
    pub position_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLivreurDto {
    #[validate(length(min = 1))]
    pub nom: String,
    #[validate(length(min = 1))]
    pub prenom: String,
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLivreurDto {
    #[validate(length(min = 1))]
    pub nom: Option<String>,
    #[validate(length(min = 1))]
    pub prenom: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub disponible: Option<bool>,
    pub position_lat: Option<f64>,
    pub position_lng: Option<f64>,
}
