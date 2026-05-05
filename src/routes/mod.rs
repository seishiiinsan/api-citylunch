use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    config::Config,
    handlers::{auth, livreur, produit, sac},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
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
        );

    let auth_routes = Router::new().route("/login", post(auth::login));

    let sac_routes = Router::new()
        .route("/", get(sac::get_sac))
        .route("/produits", post(sac::add_produit_sac))
        .route("/produits/:produit_id", delete(sac::remove_produit_sac));

    Router::new()
        .nest("/api/v1/produits", produits_routes)
        .nest("/api/v1/livreurs", livreurs_routes)
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/sac", sac_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
