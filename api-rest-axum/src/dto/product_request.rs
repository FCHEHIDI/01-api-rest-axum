// dto/product_request.rs — Objets de transfert pour les requêtes entrantes.
// Ces structs sont la SEULE porte d'entrée des données client dans l'app.
// Elles valident et contraignent les inputs AVANT qu'ils atteignent le service.

use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// Payload pour POST /products
/// Deserialize : JSON → struct (entrée réseau)
/// Validate : contraintes métier vérifiées par ValidatedJson<T>
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProductRequest {
    /// length(min=1) : rejette les strings vides "" 
    /// length(max=200) : cohérent avec CHECK SQL dans la migration
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: String,

    /// range(min=0.01) : price strictement positif
    /// On utilise f64 ici pour la simplicité de validation ;
    /// le repository convertira vers le type SQL approprié
    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price: f64,

    /// Option : champ optionnel, absent du JSON = None, pas d'erreur
    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    /// range(min=0) : stock ne peut pas être négatif
    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: i32,
}

/// Payload pour PUT /products/:id
/// Tous les champs sont Option : on ne met à jour que ce qui est fourni.
/// C'est le pattern "partial update" — plus flexible qu'un remplacement total.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 200, message = "Name must be 1-200 characters"))]
    pub name: Option<String>,

    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price: Option<f64>,

    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,

    #[validate(range(min = 0, message = "Stock cannot be negative"))]
    pub stock: Option<i32>,
}