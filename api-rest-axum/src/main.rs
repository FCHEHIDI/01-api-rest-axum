// main.rs — Point d'entrée final : câblage de toutes les couches.
// Ordre de boot : logs → config → pool → migrations → router → serveur

mod app;
mod config;
mod db;
mod dto;
mod error;
mod models;
mod repository;
mod routes;
mod services;

use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;

use app::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Logs structurés — RUST_LOG=info,tower_http=debug,sqlx=warn
    //    sqlx=warn : évite le flood des queries en dev, mais garde les erreurs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,sqlx=warn".parse().unwrap()),
        )
        .init();

    // 2. Charge .env (silencieux si absent — normal en prod)
    dotenvy::dotenv().ok();

    // 3. Config depuis l'environnement
    let config = config::Config::from_env()?;
    tracing::info!(port = config.port, env = %config.app_env, version = %config.version, "Config loaded");

    // 4. Pool PostgreSQL
    let pool = db::create_pool(&config).await?;
    tracing::info!("Database pool created");

    // 5. Migrations — idempotentes, safe à chaque démarrage
    db::run_migrations(&pool).await?;
    tracing::info!("Migrations applied");

    // 6. AppState — partagé entre tous les workers via Clone (Arc interne)
    let state = AppState {
        db: pool,
        config: Arc::new(config.clone()),
    };

    // 7. Router complet : products + health + swagger
    let app = app::create_router(state);

    // 8. Bind et démarrage du serveur
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://localhost:{}", config.port);
    tracing::info!("Swagger UI available on http://localhost:{}/docs", config.port);

    axum::serve(listener, app).await?;

    Ok(())
}