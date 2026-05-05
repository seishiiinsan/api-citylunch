use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{delete, get, post},
    Json, Router,
};
use sqlx::PgPool;
use tower_http::{
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use crate::{
    config::Config,
    handlers::{auth, livreur, produit, sac},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub fn create_router(pool: PgPool, config: Config) -> Router {
    let state = AppState { pool, config };

    let produits_routes = Router::new()
        .route(
            "/",
            get(produit::list_produits).post(produit::create_produit),
        )
        .route(
            "/:id",
            get(produit::get_produit)
                .put(produit::update_produit)
                .delete(produit::delete_produit),
        );

    let livreurs_routes = Router::new()
        .route(
            "/",
            get(livreur::list_livreurs).post(livreur::create_livreur),
        )
        .route(
            "/:id",
            get(livreur::get_livreur)
                .put(livreur::update_livreur)
                .delete(livreur::delete_livreur),
        )
        // Consultation du sac d'un livreur par le gérant
        .route("/:id/sac", get(sac::get_sac_by_livreur))
        // Historique des mouvements de stock d'un livreur
        .route("/:id/mouvements", get(sac::get_mouvements));

    let auth_routes = Router::new().route("/login", post(auth::login));

    let sac_routes = Router::new()
        .route("/", get(sac::get_sac))
        .route("/produits", post(sac::add_produit_sac))
        .route("/produits/:produit_id", delete(sac::remove_produit_sac))
        // Retour des produits en fin de service → crée des mouvements retour_fin_service
        .route("/fin-service", post(sac::fin_service));

    Router::new()
        .route("/health", get(health))
        .nest("/api/v1/produits", produits_routes)
        .nest("/api/v1/livreurs", livreurs_routes)
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/sac", sac_routes)
        // Limite la taille des payloads à 256 Ko — protège contre les DoS par payload
        .layer(DefaultBodyLimit::max(256 * 1024))
        // Headers de sécurité HTTP standard
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
