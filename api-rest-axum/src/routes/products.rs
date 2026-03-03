// routes/products.rs — Handlers HTTP pour la ressource Product.
// Chaque handler :
// 1. Extrait les données de la requête (State, Path, Query, ValidatedJson)
// 2. Délègue au service
// 3. Retourne une réponse HTTP typée

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::dto::product_request::{CreateProductRequest, UpdateProductRequest};
use crate::dto::product_response::{PageResponse, ProductResponse};
use crate::dto::validated_json::ValidatedJson;
use crate::error::AppError;
use crate::services::product_service;

/// Paramètres de query string pour GET /products
/// ?after=<uuid>&limit=<n>
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub after: Option<Uuid>,
    pub limit: Option<u32>,
}

/// Construit le sous-router pour /products
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
}

/// GET /products?after=<cursor>&limit=<n>
/// State<AppState> : Axum injecte l'état partagé automatiquement
/// Query<T> : désérialise les query params
#[utoipa::path(
    get,
    path = "/products",
    params(
        ("after" = Option<Uuid>, Query, description = "Cursor UUID"),
        ("limit" = Option<u32>, Query, description = "Max results (default 20)")
    ),
    responses(
        (status = 200, body = PageResponse<ProductResponse>),
        (status = 500, description = "Internal error")
    ),
    tag = "products"
)]

pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let page: PageResponse<ProductResponse> =
        product_service::list_products(&state.db, params.after, params.limit).await?;
    Ok(Json(page))
}

/// GET /products/:id
/// Path<Uuid> : Axum parse et valide le segment UUID automatiquement
/// Si l'UUID est malformé → 400 avant même d'atteindre le handler

#[utoipa::path(get, path = "/products/{id}",
    responses((status = 200, body = ProductResponse), (status = 404, description = "Not found")),
    tag = "products"
)]

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let product: ProductResponse = product_service::get_product(&state.db, id).await?;
    Ok(Json(product))
}

/// POST /products
/// ValidatedJson<T> : notre extractor custom — désérialise + valide en une passe
/// StatusCode::CREATED (201) : convention REST pour une création réussie
#[utoipa::path(post, path = "/products", request_body = CreateProductRequest,
    responses((status = 201, body = ProductResponse), (status = 422, description = "Validation error")),
    tag = "products"
)]

pub async fn create_product(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<CreateProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    let product: ProductResponse = product_service::create_product(&state.db, req).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

/// PUT /products/:id
#[utoipa::path(put, path = "/products/{id}", request_body = UpdateProductRequest,
    responses((status = 200, body = ProductResponse), (status = 404, description = "Not found")),
    tag = "products"
)]

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateProductRequest>,
) -> Result<impl IntoResponse, AppError> {
    let product: ProductResponse = product_service::update_product(&state.db, id, req).await?;
    Ok(Json(product))
}

/// DELETE /products/:id
/// StatusCode::NO_CONTENT (204) : convention REST pour suppression réussie sans body
#[utoipa::path(delete, path = "/products/{id}",
    responses((status = 204, description = "Deleted"), (status = 404, description = "Not found")),
    tag = "products"
)]

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    product_service::delete_product(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}