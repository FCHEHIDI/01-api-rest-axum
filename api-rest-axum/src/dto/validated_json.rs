// dto/validated_json.rs — Extractor Axum qui combine désérialisation + validation.
// Pattern : on crée un newtype wrapper autour de T.
// Axum appelle from_request() automatiquement quand un handler déclare
// ValidatedJson<T> comme paramètre.

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// Newtype wrapper : ValidatedJson<T> est un T validé.
/// Le tuple struct avec un champ public permet d'extraire la valeur
/// avec let ValidatedJson(payload) = ... dans les handlers.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    // T doit pouvoir être désérialisé depuis JSON
    T: DeserializeOwned + Validate,
    // S est l'état Axum — Send + Sync requis pour être thread-safe
    S: Send + Sync,
{
    // Si la validation échoue, on retourne notre AppError (pas un type Axum générique)
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Étape 1 : désérialiser le JSON avec l'extractor natif Axum
        // Si le JSON est malformé → AppError::Validation
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Étape 2 : valider les contraintes #[validate(...)]
        // Si une contrainte est violée → AppError::Validation avec le détail
        value
            .validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        Ok(ValidatedJson(value))
    }
}