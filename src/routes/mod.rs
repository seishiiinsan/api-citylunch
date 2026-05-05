use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::{
    cors::CorsLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Config,
    errors::ErrorResponse,
    handlers::{auth, livreur, produit, sac},
    models::{
        livreur::{CreateLivreurDto, LivreurResponse, UpdateLivreurDto},
        mouvement_stock::{MouvementResponse, MouvementsListResponse, TypeMouvement},
        produit::{CreateProduitDto, Produit, TypeProduit, UpdateProduitDto},
        sac::{AddProduitDto, FinServiceResponse, RetraitProduitDto, SacItemResponse, SacResponse},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
}

#[derive(Serialize, utoipa::ToSchema)]
struct HealthResponse {
    #[schema(example = "ok")]
    status: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "santé",
    responses(
        (status = 200, description = "API opérationnelle", body = HealthResponse),
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok".to_string() })
}

/// Modificateur qui ajoute le schéma de sécurité bearerAuth (JWT) à la spec OpenAPI.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        produit::list_produits,
        produit::get_produit,
        produit::create_produit,
        produit::update_produit,
        produit::delete_produit,
        livreur::list_livreurs,
        livreur::get_livreur,
        livreur::create_livreur,
        livreur::update_livreur,
        livreur::delete_livreur,
        sac::get_sac_by_livreur,
        sac::get_mouvements,
        auth::login,
        sac::get_sac,
        sac::add_produit_sac,
        sac::remove_produit_sac,
        sac::fin_service,
    ),
    components(schemas(
        HealthResponse,
        Produit,
        TypeProduit,
        CreateProduitDto,
        UpdateProduitDto,
        LivreurResponse,
        CreateLivreurDto,
        UpdateLivreurDto,
        SacResponse,
        SacItemResponse,
        AddProduitDto,
        RetraitProduitDto,
        FinServiceResponse,
        MouvementsListResponse,
        MouvementResponse,
        TypeMouvement,
        auth::LoginDto,
        auth::LoginResponse,
        ErrorResponse,
    )),
    tags(
        (name = "produits",  description = "Gestion des produits"),
        (name = "livreurs",  description = "Gestion des livreurs"),
        (name = "auth",      description = "Authentification JWT"),
        (name = "sac",       description = "Gestion du sac du livreur (JWT requis)"),
        (name = "santé",     description = "Health check"),
    ),
    info(
        title = "CityLunch API",
        version = "1.0.0",
        description = "API REST pour la gestion des livreurs, produits et sacs CityLunch.\n\nLes routes **sac** (POST /sac/produits, DELETE /sac/produits/:id, GET /sac, POST /sac/fin-service) nécessitent un token JWT dans le header `Authorization: Bearer <token>`."
    ),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
