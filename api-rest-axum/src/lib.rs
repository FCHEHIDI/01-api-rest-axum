// lib.rs — Crate bibliothèque exposé aux tests d'intégration (tests/).
//
// Le binaire (main.rs) déclare ses propres `mod` en privé.
// Le crate lib déclare les mêmes modules en `pub` pour que les tests
// dans le dossier `tests/` puissent importer `api_rest_axum::app`, etc.

pub mod app;
pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;
pub mod services;
