// error.rs — Le pattern central d'une API Axum production-ready.
// Principe : UN seul enum d'erreurs couvre tous les cas possibles.
// Chaque variant se mappe vers un status HTTP précis + un body JSON cohérent.
// Aucun handler n'appelle .unwrap() — tout remonte via ? vers AppError.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Ressource introuvable : GET /products/:id avec un id inexistant → 404
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Payload invalide : champ manquant, type incorrect, contrainte violée → 422
    /// 422 Unprocessable Entity est plus précis que 400 Bad Request :
    /// la syntaxe JSON est valide, mais la sémantique est incorrecte.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Conflit de données : ex. nom de produit déjà existant → 409
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Erreur PostgreSQL — le #[from] permet d'utiliser ? directement
    /// sur les résultats sqlx::Result sans .map_err() manuel
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    /// Erreur générique non anticipée — #[from] anyhow::Error pour
    /// les fonctions qui retournent anyhow::Result
    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

// Body JSON standardisé pour TOUTES les erreurs de l'API.
// `code` : constante machine-readable (pour les clients API)
// `message` : message humain (pour le debug)
// Exemple : {"code":"NOT_FOUND","message":"Product abc-123 not found"}
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

fn error_body(code: &'static str, msg: &str) -> ErrorBody {
    ErrorBody {
        code,
        message: msg.to_owned(),
    }
}

// IntoResponse est le trait Axum qui transforme une valeur en réponse HTTP.
// En implémentant ce trait sur AppError, on peut écrire dans les handlers :
//   return Err(AppError::NotFound("product not found".into()));
// et Axum se charge d'appeler into_response() automatiquement.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                error_body("NOT_FOUND", msg),
            ),
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                error_body("VALIDATION_ERROR", msg),
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                error_body("CONFLICT", msg),
            ),
            AppError::Database(e) => {
                // On logue l'erreur interne mais on NE l'expose PAS au client.
                // Le client voit "internal error", les logs contiennent le détail.
                // Sécurité : ne jamais leaker les messages d'erreur DB en prod.
                tracing::error!(error = %e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_body("DB_ERROR", "internal error"),
                )
            }
            AppError::Internal(e) => {
                tracing::error!(error = ?e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_body("INTERNAL_ERROR", "internal error"),
                )
            }
        };

        // (StatusCode, Json<T>) implémente IntoResponse nativement dans Axum
        (status, Json(body)).into_response()
    }
}