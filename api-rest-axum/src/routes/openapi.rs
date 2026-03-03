// routes/openapi.rs — Génération automatique de la doc OpenAPI 3.0
// utoipa scanne les annotations #[utoipa::path] sur les handlers
// et génère le JSON OpenAPI sans maintenir un fichier YAML à la main.

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::dto::product_request::{CreateProductRequest, UpdateProductRequest};
use crate::dto::product_response::{PageResponse, ProductResponse};

// #[derive(OpenApi)] génère la struct ApiDoc avec la méthode openapi()
// qui retourne le JSON OpenAPI complet.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "API REST Axum",
        version = "0.1.0",
        description = "API de gestion de produits — production-ready avec Axum"
    ),
    paths(
        // On référence les handlers par leur chemin de module
        crate::routes::products::list_products,
        crate::routes::products::get_product,
        crate::routes::products::create_product,
        crate::routes::products::update_product,
        crate::routes::products::delete_product,
    ),
    components(
        schemas(
            ProductResponse,
            CreateProductRequest,
            UpdateProductRequest,
            PageResponse<ProductResponse>,
        )
    ),
    tags(
        (name = "products", description = "Gestion des produits")
    )
)]
pub struct ApiDoc;

/// Retourne le SwaggerUI monté sur /docs
/// SwaggerUi::new("/docs") : URL de l'interface
/// .url("/api-doc/openapi.json", ApiDoc::openapi()) : endpoint JSON brut
pub fn swagger_router() -> SwaggerUi {
    SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi())
}