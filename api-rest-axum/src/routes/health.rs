// routes/health.rs — Endpoint de santé, indispensable en prod.
// Utilisé par les load balancers (AWS ALB, nginx) et les orchestrateurs (Kubernetes)
// pour savoir si l'instance est prête à recevoir du trafic.
// Retourne 200 si tout va bien, 503 si la DB est inaccessible.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::app::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub db: &'static str,
    pub version: String,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Ping léger vers PostgreSQL — vérifie que le pool a une connexion active
    let db_status = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await;

    match db_status {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                db: "ok",
                version: state.config.version.clone(),
            }),
        ),
        Err(_) => (
            // 503 Service Unavailable : le load balancer retire l'instance du pool
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "degraded",
                db: "unreachable",
                version: state.config.version.clone(),
            }),
        ),
    }
}