// services/product_service.rs — Logique métier de la ressource Product.
// Cette couche :
// 1. Orchestre les appels au repository
// 2. Convertit les résultats en DTOs (Product → ProductResponse)
// 3. Mappe les erreurs repository → AppError métier (None → NotFound, etc.)
// 4. Calcule next_cursor pour la pagination

use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::product_request::{CreateProductRequest, UpdateProductRequest};
use crate::dto::product_response::{PageResponse, ProductResponse};
use crate::error::AppError;
use crate::repository::product_repo;

/// Liste les produits avec pagination cursor-based.
/// Retourne un PageResponse<ProductResponse> prêt à sérialiser.
pub async fn list_products(
    pool: &PgPool,
    after: Option<Uuid>,
    limit: Option<u32>,
) -> Result<PageResponse<ProductResponse>, AppError> {
    // Applique les bornes : défaut 20, max 100
    // Clamping côté service, pas côté handler : logique métier ici.
    let limit = limit.unwrap_or(20).min(100) as i64;

    // On lance les deux queries en parallèle avec tokio::try_join!
    // try_join! : attend les deux, retourne Err dès que l'une échoue.
    // Plus efficace que deux .await séquentiels (latence réseau DB en parallèle).
    let (products, total_count) = tokio::try_join!(
        product_repo::find_all(pool, after, limit),
        product_repo::count_all(pool),
    )?;
    // Le ? convertit sqlx::Error → AppError::Database via #[from]

    // Le cursor pour la page suivante = id du dernier élément retourné.
    // Si on a reçu exactement `limit` produits, il y a probablement une page suivante.
    // Si on en a reçu moins, on est sur la dernière page → next_cursor = None.
    let next_cursor = if products.len() == limit as usize {
        products.last().map(|p| p.id)
    } else {
        None
    };

    // Convertit Vec<Product> → Vec<ProductResponse> via l'impl From qu'on a écrit
    let data = products.into_iter().map(ProductResponse::from).collect();

    Ok(PageResponse {
        data,
        next_cursor,
        total_count,
    })
}

/// Récupère un produit par id ou retourne NotFound.
pub async fn get_product(
    pool: &PgPool,
    id: Uuid,
) -> Result<ProductResponse, AppError> {
    product_repo::find_by_id(pool, id)
        .await?                          // sqlx::Error → AppError::Database
        .map(ProductResponse::from)      // Product → ProductResponse
        // None → AppError::NotFound avec un message explicite
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))
}

/// Crée un nouveau produit.
pub async fn create_product(
    pool: &PgPool,
    req: CreateProductRequest,
) -> Result<ProductResponse, AppError> {
    let product = product_repo::insert(pool, &req).await?;
    Ok(ProductResponse::from(product))
}

/// Met à jour un produit existant (partial update).
pub async fn update_product(
    pool: &PgPool,
    id: Uuid,
    req: UpdateProductRequest,
) -> Result<ProductResponse, AppError> {
    product_repo::update(pool, id, &req)
        .await?
        .map(ProductResponse::from)
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))
}

/// Supprime un produit ou retourne NotFound.
pub async fn delete_product(
    pool: &PgPool,
    id: Uuid,
) -> Result<(), AppError> {
    let deleted = product_repo::delete(pool, id).await?;
    if deleted {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("Product {} not found", id)))
    }
}