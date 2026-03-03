// tests/integration_test.rs — Tests d'intégration CP11
//
// Stratégie :
//   #[sqlx::test(migrations = "./migrations")]
//     → crée une BDD PostgreSQL temporaire vide par test
//     → applique les migrations
//     → injecte un PgPool pointant vers cette BDD
//     → supprime la BDD après le test (isolation totale)
//
//   axum_test::TestServer
//     → monte le router Axum in-process (pas de socket réseau)
//     → API fluide : server.post("/products").json(&payload).await
//
// Pré-requis : DATABASE_URL doit pointer vers le serveur Postgres
//   (le chemin /db est remplacé par une BDD temporaire par le macro).
//   Charger via .env ou variable d'environnement shell avant `cargo test`.

use api_rest_axum::app::{AppState, create_router};
use api_rest_axum::config::Config;
use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

// ─── Helper ─────────────────────────────────────────────────────────────────

/// Construit un TestServer in-process à partir d'un pool de test.
/// Le pool est fourni par #[sqlx::test] — il pointe vers une BDD temporaire.
fn make_server(pool: PgPool) -> TestServer {
    // Charge .env pour DATABASE_URL si pas encore dans l'environnement
    dotenvy::dotenv().ok();

    let state = AppState {
        db: pool,
        config: Arc::new(Config::for_test()),
    };
    TestServer::new(create_router(state)).expect("Failed to start TestServer")
}

// ─── /health ────────────────────────────────────────────────────────────────

/// GET /health → 200 avec {"status":"ok","db":"ok","version":"0.1.0"}
#[sqlx::test(migrations = "./migrations")]
async fn health_returns_ok(pool: PgPool) {
    let server = make_server(pool);

    let response = server.get("/health").await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["db"], "ok");
}

// ─── POST /products ──────────────────────────────────────────────────────────

/// Création valide → 201 avec le produit retourné (id, name, price, stock…)
#[sqlx::test(migrations = "./migrations")]
async fn create_product_valid(pool: PgPool) {
    let server = make_server(pool);

    let response = server
        .post("/products")
        .json(&json!({
            "name": "Widget Premium",
            "description": "Un widget de haute qualité",
            "price": 49.99,
            "stock": 100
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["name"], "Widget Premium");
    assert_eq!(body["price"], 49.99);
    assert_eq!(body["stock"], 100);
    assert!(body["id"].is_string(), "id doit être présent");
}

/// Nom vide → 422 VALIDATION_ERROR
#[sqlx::test(migrations = "./migrations")]
async fn create_product_name_empty(pool: PgPool) {
    let server = make_server(pool);

    let response = server
        .post("/products")
        .json(&json!({
            "name": "",
            "price": 10.0,
            "stock": 5
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = response.json();
    assert_eq!(body["code"], "VALIDATION_ERROR");
}

/// Prix négatif → 422 VALIDATION_ERROR
#[sqlx::test(migrations = "./migrations")]
async fn create_product_price_negative(pool: PgPool) {
    let server = make_server(pool);

    let response = server
        .post("/products")
        .json(&json!({
            "name": "Produit invalide",
            "price": -5.0,
            "stock": 10
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = response.json();
    assert_eq!(body["code"], "VALIDATION_ERROR");
}

/// Corps JSON manquant → 422 (Axum rejette avant nos validators)
#[sqlx::test(migrations = "./migrations")]
async fn create_product_missing_body(pool: PgPool) {
    let server = make_server(pool);

    let response = server.post("/products").await;

    // Axum retourne 415 Unsupported Media Type sans Content-Type: application/json
    // ou 422 si le corps est absent — on accepte les deux codes d'échec
    let status = response.status_code();
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "attendu 422 ou 415, reçu {}",
        status
    );
}

// ─── GET /products ───────────────────────────────────────────────────────────

/// BDD vide → 200 data=[] total_count=0
#[sqlx::test(migrations = "./migrations")]
async fn list_products_empty(pool: PgPool) {
    let server = make_server(pool);

    let response = server.get("/products").await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["data"], json!([]));
    assert_eq!(body["total_count"], 0);
    assert!(body["next_cursor"].is_null());
}

/// Après création de 2 produits → total_count=2, data.len()=2
#[sqlx::test(migrations = "./migrations")]
async fn list_products_returns_all(pool: PgPool) {
    let server = make_server(pool);

    // Seed : 2 produits
    for (name, price) in [("Produit A", 10.0), ("Produit B", 20.0)] {
        server
            .post("/products")
            .json(&json!({ "name": name, "price": price, "stock": 1 }))
            .await
            .assert_status(StatusCode::CREATED);
    }

    let response = server.get("/products").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["total_count"], 2);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ─── GET /products/:id ───────────────────────────────────────────────────────

/// Produit existant → 200 avec les champs corrects
#[sqlx::test(migrations = "./migrations")]
async fn get_product_found(pool: PgPool) {
    let server = make_server(pool);

    // Crée un produit et récupère son id
    let created: Value = server
        .post("/products")
        .json(&json!({ "name": "Produit X", "price": 99.0, "stock": 7 }))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.get(&format!("/products/{}", id)).await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Produit X");
    assert_eq!(body["stock"], 7);
}

/// UUID inexistant → 404 avec {"code":"NOT_FOUND","message":"..."}
#[sqlx::test(migrations = "./migrations")]
async fn get_product_not_found(pool: PgPool) {
    let server = make_server(pool);
    let fake_id = "00000000-0000-0000-0000-000000000001";

    let response = server.get(&format!("/products/{}", fake_id)).await;

    response.assert_status_not_found();
    let body: Value = response.json();
    assert_eq!(body["code"], "NOT_FOUND");
    assert!(body["message"].as_str().unwrap().contains(fake_id));
}

// ─── PUT /products/:id ───────────────────────────────────────────────────────

/// Mise à jour partielle (stock seul) → 200 avec updated_at changé
#[sqlx::test(migrations = "./migrations")]
async fn update_product_partial(pool: PgPool) {
    let server = make_server(pool);

    let created: Value = server
        .post("/products")
        .json(&json!({ "name": "Original", "price": 50.0, "stock": 10 }))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/products/{}", id))
        .json(&json!({ "stock": 42 }))
        .await;

    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["stock"], 42);
    // Le nom ne doit pas avoir changé
    assert_eq!(body["name"], "Original");
    // updated_at doit être présent
    assert!(body["updated_at"].is_string());
}

// ─── DELETE /products/:id ────────────────────────────────────────────────────

/// Suppression → 204, puis GET → 404 NOT_FOUND
#[sqlx::test(migrations = "./migrations")]
async fn delete_product_then_not_found(pool: PgPool) {
    let server = make_server(pool);

    let created: Value = server
        .post("/products")
        .json(&json!({ "name": "À supprimer", "price": 1.0, "stock": 1 }))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    // Suppression
    server
        .delete(&format!("/products/{}", id))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // GET après suppression → 404
    let response = server.get(&format!("/products/{}", id)).await;
    response.assert_status_not_found();
    let body: Value = response.json();
    assert_eq!(body["code"], "NOT_FOUND");
}
