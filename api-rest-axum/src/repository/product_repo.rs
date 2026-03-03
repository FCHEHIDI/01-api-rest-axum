// repository/product_repo.rs — Toutes les queries SQLx pour la table products.
// Règle : cette couche ne contient QUE des accès DB, zéro logique métier.
// Chaque fonction prend un &PgPool et retourne Result<_, sqlx::Error>.
// Le service au-dessus convertira sqlx::Error → AppError via #[from].

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::product::Product;
use crate::dto::product_request::{CreateProductRequest, UpdateProductRequest};

/// Récupère tous les produits avec pagination cursor-based.
/// 
/// Cursor-based vs offset/limit :
/// - OFFSET 1000 : PostgreSQL scanne et jette les 1000 premières lignes → lent
/// - WHERE id > $cursor : utilise l'index PK directement → O(log n) constant
/// 
/// `after` = dernier UUID vu par le client (le "cursor")
/// `limit` = nombre de résultats à retourner
pub async fn find_all(
    pool: &PgPool,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<Product>, sqlx::Error> {
    // r#"..."# = raw string literal : pas besoin d'échapper les guillemets
    // $1::uuid : cast explicite nécessaire car $1 peut être NULL
    sqlx::query_as!(
        Product,
        r#"SELECT id, name, description, price, stock, created_at, updated_at
           FROM products
           WHERE ($1::uuid IS NULL OR id > $1)
           ORDER BY id ASC
           LIMIT $2"#,
        after as Option<Uuid>,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// Compte le total de produits — pour le champ total_count de PageResponse.
/// Query séparée : plus simple qu'un COUNT dans la même query paginée.
pub async fn count_all(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!("SELECT COUNT(*) as count FROM products")
        .fetch_one(pool)
        .await?;
    // COUNT(*) retourne Option<i64> dans sqlx — unwrap_or(0) : safe car
    // COUNT ne retourne jamais NULL sur une table existante
    Ok(row.count.unwrap_or(0))
}

/// Récupère un produit par son UUID.
/// fetch_optional : retourne None si pas trouvé (pas d'erreur).
/// Le service convertira None → AppError::NotFound.
pub async fn find_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"SELECT id, name, description, price, stock, created_at, updated_at
           FROM products
           WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Insère un nouveau produit et retourne la ligne créée.
/// RETURNING * : récupère le produit avec id et timestamps générés par PostgreSQL.
/// Sans RETURNING, il faudrait faire un second SELECT — moins efficace.
pub async fn insert(
    pool: &PgPool,
    req: &CreateProductRequest,
) -> Result<Product, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"INSERT INTO products (name, description, price, stock)
           VALUES ($1, $2, $3, $4)
           RETURNING id, name, description, price, stock, created_at, updated_at"#,
        req.name,
        req.description,
        req.price,
        req.stock,
    )
    .fetch_one(pool)
    .await
}

/// Met à jour un produit existant avec les champs fournis (partial update).
/// COALESCE($1, name) : si $1 est NULL (champ absent du payload),
/// conserve la valeur actuelle de la colonne — pattern partial update.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    req: &UpdateProductRequest,
) -> Result<Option<Product>, sqlx::Error> {
    sqlx::query_as!(
        Product,
        r#"UPDATE products
           SET name        = COALESCE($1, name),
               description = COALESCE($2, description),
               price       = COALESCE($3, price),
               stock       = COALESCE($4, stock)
           WHERE id = $5
           RETURNING id, name, description, price, stock, created_at, updated_at"#,
        req.name.clone() as Option<String>,
        req.description.clone() as Option<String>,
        req.price as Option<f64>,
        req.stock as Option<i32>,
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Supprime un produit.
/// rows_affected() : 0 = produit introuvable, 1 = supprimé.
/// On retourne un bool pour que le service puisse déclencher NotFound si 0.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM products WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}