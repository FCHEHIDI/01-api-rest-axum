// dto/product_response.rs — Objets de transfert pour les réponses sortantes.
// On ne renvoie JAMAIS le modèle DB directement :
// 1. On contrôle exactement ce qui est exposé (pas de champs internes)
// 2. On peut formater les types différemment (OffsetDateTime → string ISO 8601)

use serde::Serialize;
use uuid::Uuid;

/// Représentation d'un produit dans les réponses HTTP.
/// Différence avec Product (modèle DB) : created_at/updated_at en String
/// pour un format ISO 8601 lisible côté client.
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Réponse paginée générique — fonctionne pour n'importe quel type T.
/// Le générique <T> évite de réécrire cette struct pour chaque ressource.
#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    /// None = on est sur la dernière page
    pub next_cursor: Option<Uuid>,
    /// Nombre total de produits en DB (pour afficher "Page 1 sur N")
    pub total_count: i64,
}

use crate::models::product::Product;

impl From<Product> for ProductResponse {
    /// Conversion Product (DB) → ProductResponse (HTTP).
    /// Ce `From` permet d'écrire : ProductResponse::from(product)
    /// ou product.into() dans les handlers — zéro boilerplate.
    fn from(p: Product) -> Self {
        ProductResponse {
            id: p.id,
            name: p.name,
            description: p.description,
            price: p.price,
            stock: p.stock,
            // format_into() de `time` : ISO 8601 → "2026-03-03T14:46:08+00:00"
            created_at: p.created_at.to_string(),
            updated_at: p.updated_at.to_string(),
        }
    }
}