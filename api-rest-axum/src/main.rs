// main.rs — Point d'entrée minimal pour valider le CP2
// On va progressivement enrichir ce fichier jusqu'au CP10.

mod config;
mod db;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialise les logs — RUST_LOG=debug pour voir les requêtes SQLx
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
        )
        .init();

    // 2. Charge .env en développement (no-op si le fichier est absent)
    dotenvy::dotenv().ok();

    // 3. Charge la config — erreur fatale si une variable est manquante
    let config = config::Config::from_env()?;
    tracing::info!(port = config.port, env = %config.app_env, "Config loaded");

    // 4. Crée le pool PostgreSQL
    let pool = db::create_pool(&config).await?;
    tracing::info!("Database pool created");

    // 5. Lance les migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Migrations done — server ready on port {}", config.port);

    Ok(())
}