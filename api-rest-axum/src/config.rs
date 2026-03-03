// config.rs — Lecture de la configuration depuis l'environnement
// Pattern : struct immuable chargée une seule fois au démarrage,
// puis partagée via Arc<Config> dans AppState.

use std::env;
use anyhow::{Context, Result};

/// Configuration globale de l'application.
/// Tous les champs sont lus depuis les variables d'environnement. 
/// On utilise `anyhow::Context` pour enrichir les messages d'erreur.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL complète PostgresSSQL - jamais hardcodée, toujours depuis l'env
    pub database_url: String,

    /// Port d'écoute du serveur HTTP
    pub port: u16,

    /// Environnement : "dévelopment" | "production" | "test"
    pub app_env: String,

    /// Version de l'application - lue depuis Cargo.toml via env! macro
    pub version: String,
}

impl Config {
    /// Charge la config depuis les variables d'environnement.
    /// A appeler UNE SEULE FOIS dans main(), après dotenvy::dotenv().
    /// Retourne une erreur descriptive si une variable est manquante.
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
            // context() : si la variable est absente, l'erreur contiendra
            // "DATABASE_URL must be set" au lieu du générique VarError
            .context("DATABASE_URL must be set")?,
            
        port: env::var("APP_PORT")
            .context("APP_PORT must be set")?
            .parse::<u16>()
            // On enrichit l'erreur de parse avec le contexte
            .context("APP_PORT must be a valid port number (1-65535)")?,

        app_env: env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string()),
        // unwrap_or_else : APP_ENV est optionnel, "development" par défaut

        // env!() est une macro Rust évaluée à la COMPILATION
        // Elle lit CARGO_PKG_VERSION depuis Cargo.toml - pas d'allocation runtime
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    // Raccourci pour savoir si on est en test (utile pour les logs)
    pub fn is_test(&self) -> bool {
        self.app_env == "test"
    }
}
