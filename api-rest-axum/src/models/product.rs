// models/product.rs — Struct qui mappe EXACTEMENT la table PostgreSQL
// Règle fondamentale : les types Rust doivent correspondre aux types SQL.
// SQLx vérifie cette correspondance à la compilation via query_as!

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// Représentation d'un produit tel qu'il existe en base de données.
/// 
/// `#[derive(FromRow)]` : SQLx peut mapper une row PostgreSQL vers cette struct
/// automatiquement — les noms de champs doivent correspondre aux noms de colonnes.
/// 
/// `#[derive(Serialize)]` : permet de sérialiser en JSON pour les réponses HTTP.
/// On ne dérive PAS Deserialize ici — on ne désérialise jamais directement
/// depuis une requête HTTP vers le modèle DB. On passe toujours par les DTOs.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Product {
    pub id: Uuid,

    pub name: String,

    /// Option<String> car la colonne SQL est nullable (TEXT sans NOT NULL)
    /// Si on mettait String ici, query_as! refuserait de compiler
    pub description: Option<String>,

    /// NUMERIC(10,2) en SQL → sqlx::types::BigDecimal ou f64 en Rust.
    /// On utilise f64 pour la simplicité ; en prod financière on utiliserait
    /// rust_decimal::Decimal pour éviter les erreurs d'arrondi IEEE 754.
    pub price: f64,

    pub stock: i32,

    /// TIMESTAMPTZ PostgreSQL → OffsetDateTime du crate `time`
    /// OffsetDateTime inclut le timezone offset — safe pour les comparaisons
    pub created_at: OffsetDateTime,

    pub updated_at: OffsetDateTime,
}