// app.rs — Assemblage du router Axum et de l'état partagé.
// AppState est cloné pour chaque worker thread — d'où le Arc<Config> :
// Arc permet le partage sans copie, RwLock si on avait du mutable partagé.

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::routes;

/// État partagé injecté dans chaque handler via axum::extract::State.
/// Clone est peu coûteux : PgPool est déjà un Arc interne, Arc<Config> est un pointeur.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

/// Construit le router complet de l'application.
/// Appelé une seule fois dans main(), puis passé au serveur Axum.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Toutes les routes /products sont définies dans routes::products
        .merge(routes::products::router())
        // TraceLayer : log automatique de chaque requête HTTP (method, path, status, latence)
        // Visible avec RUST_LOG=tower_http=debug
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}