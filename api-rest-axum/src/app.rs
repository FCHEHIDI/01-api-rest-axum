// app.rs — Assemblage du router Axum et de l'état partagé.
// AppState est cloné pour chaque worker thread — d'où le Arc<Config> :
// Arc permet le partage sans copie, RwLock si on avait du mutable partagé.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::routes;
use crate::routes::products::{
    create_product, delete_product, get_product, list_products, update_product,
};

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
    // Typage explicite Router<AppState> AVANT with_state() :
    // garantit que tous les handlers peuvent extraire State<AppState>.
    let router: Router<AppState> = Router::new()
        // IMPORTANT: Axum 0.7 (matchit 0.7) utilise :id, PAS {id}.
        // La syntaxe {id} (accolades) n'est disponible qu'à partir d'Axum 0.8.
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/health", get(routes::health::health_check))
        .merge(routes::openapi::swagger_router())
        .layer(TraceLayer::new_for_http());

    router.with_state(state)
}