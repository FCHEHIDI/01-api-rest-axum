// db.rs — Création et configuration du pool de connexions PostgreSQL
// Le pool est la ressource la plus critique de l'app :
// mal configuré → timeouts en prod, connexions fantômes, memory leaks.

use std::time::Duration;
use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::Config;

/// Crée le pool SQLx avec des paramètres de production.
/// 
/// # Pourquoi ces valeurs ?
/// - `max_connections(10)` : PostgreSQL a une limite (~100 max par défaut).
///   10 connexions par instance d'app laisse de la marge pour plusieurs replicas.
/// - `acquire_timeout(30s)` : si le pool est saturé, on attend 30s avant d'envoyer
///   une erreur 503 — évite d'empiler les requêtes indéfiniment.
/// - `idle_timeout(600s)` : ferme les connexions inactives après 10 min.
///   Évite que PostgreSQL ferme les connexions de son côté (timeout serveur).
/// - `min_connections(1)` : garde 1 connexion "chaude" — le premier appel après
///   un démarrage à froid ne souffre pas d'une latence de handshake TLS.
pub async fn create_pool(config: &Config) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .min_connections(1)
        .connect(&config.database_url)
        .await
        // .context() encapsule l'erreur sqlx dans une anyhow::Error
        // avec un message lisible dans les logs de démarrage
        .context("Failed to connect to PostgreSQL")
}

/// Lance les migrations SQLx versionnées depuis le dossier `migrations/`.
///
/// SQLx maintient une table `_sqlx_migrations` en DB pour tracker
/// quelles migrations ont déjà été jouées - idempotent et safe en prod. 
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Database migrations applied successfully");
    Ok(())
}