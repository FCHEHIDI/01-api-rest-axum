# api-rest-axum

API REST production-ready en Rust avec **Axum 0.7**, **SQLx 0.8** et **PostgreSQL**.  
Architecture en couches, pagination cursor-based, gestion d'erreurs typées, validation automatique, documentation OpenAPI, tests d'intégration avec base de données réelle.

---

## Stack

| Crate | Rôle |
|---|---|
| `axum 0.7` | Framework HTTP (Tokio-native) |
| `sqlx 0.8` | Requêtes async type-checkées à la compilation |
| `tokio 1` | Runtime async multi-thread |
| `tower-http 0.6` | Middleware : `TraceLayer`, `CorsLayer` |
| `serde` / `serde_json` | Sérialisation JSON |
| `validator 0.20` | Validation des payloads entrants |
| `thiserror 2` | `AppError` typé avec `IntoResponse` |
| `utoipa 4` + `utoipa-swagger-ui 7` | Documentation OpenAPI générée automatiquement |
| `axum-test 16` | Driver HTTP pour les tests d'intégration |

---

## Prérequis

- Rust stable (≥ 1.75)
- Docker (pour PostgreSQL)
- `sqlx-cli` : `cargo install sqlx-cli --no-default-features --features postgres`

---

## Démarrage rapide

```bash
# 1. Démarrer PostgreSQL
docker run -d --name pg-axum \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres123 \
  -e POSTGRES_DB=api_rest_axum \
  -p 5432:5432 \
  postgres:17-alpine

# 2. Variables d'environnement
cp .env.example .env   # ou créer .env manuellement

# 3. Migrations
sqlx database create
sqlx migrate run

# 4. Lancer le serveur
cargo run
```

Serveur disponible sur `http://localhost:3000`  
Swagger UI sur `http://localhost:3000/docs`

---

## Fichier `.env`

```env
DATABASE_URL=postgres://postgres:postgres123@localhost:5432/api_rest_axum
APP_PORT=3000
APP_ENV=development
APP_VERSION=0.1.0
RUST_LOG=tower_http=debug,info
```

---

## Endpoints

| Méthode | Route | Description | Status |
|---|---|---|---|
| `GET` | `/health` | État du serveur + DB | 200 |
| `GET` | `/products` | Liste paginée (cursor-based) | 200 |
| `POST` | `/products` | Créer un produit | 201 |
| `GET` | `/products/:id` | Récupérer par UUID | 200 / 404 |
| `PUT` | `/products/:id` | Mise à jour partielle | 200 / 404 |
| `DELETE` | `/products/:id` | Supprimer | 204 / 404 |
| `GET` | `/docs` | Swagger UI | 200 |
| `GET` | `/api-doc/openapi.json` | Spec OpenAPI 3.0 brute | 200 |

### Pagination cursor-based

```
GET /products?limit=20&after=<uuid>
```

```json
{
  "data": [...],
  "next_cursor": "550e8400-e29b-41d4-a716-446655440000",
  "total_count": 142
}
```

`next_cursor` est `null` sur la dernière page. Plus performant qu'OFFSET : pas de scan complet, stable sous insertion concurrente.

### Exemple POST

```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"name":"Widget","price":29.99,"stock":100}'
```

### Codes d'erreur

Toutes les erreurs retournent un JSON structuré :

```json
{ "code": "NOT_FOUND", "message": "Product abc... not found" }
```

| Code | HTTP | Déclencheur |
|---|---|---|
| `NOT_FOUND` | 404 | UUID inexistant |
| `VALIDATION_ERROR` | 422 | Payload invalide (nom vide, prix négatif…) |
| `CONFLICT` | 409 | Contrainte d'unicité |
| `DB_ERROR` | 500 | Erreur SQLx |
| `INTERNAL_ERROR` | 500 | Erreur interne |

---

## Structure

```
api-rest-axum/
├── migrations/
│   ├── 001_create_products.sql   ← table + trigger updated_at
│   └── 002_add_indexes.sql       ← index name, created_at, price
├── src/
│   ├── main.rs                   ← boot : config → pool → migrations → serve
│   ├── lib.rs                    ← exports pub pour les tests d'intégration
│   ├── app.rs                    ← AppState + create_router()
│   ├── config.rs                 ← Config depuis variables d'env
│   ├── db.rs                     ← PgPool + migrations helper
│   ├── error.rs                  ← AppError → IntoResponse
│   ├── models/product.rs         ← struct Product (FromRow)
│   ├── dto/
│   │   ├── product_request.rs    ← CreateProductRequest, UpdateProductRequest
│   │   ├── product_response.rs   ← ProductResponse, PageResponse<T>
│   │   └── validated_json.rs     ← ValidatedJson<T> extractor
│   ├── repository/product_repo.rs ← queries SQLx
│   ├── services/product_service.rs ← logique métier + tokio::try_join!
│   └── routes/
│       ├── mod.rs
│       ├── products.rs           ← 5 handlers HTTP
│       ├── health.rs             ← GET /health
│       └── openapi.rs            ← ApiDoc utoipa + SwaggerUi
└── tests/
    └── integration_test.rs       ← 11 tests, DB réelle par test
```

---

## Tests

```bash
# Nécessite DATABASE_URL dans l'environnement
cargo test
```

`#[sqlx::test]` crée une base de données isolée pour chaque test, exécute les migrations, puis supprime la base à la fin. Aucun mock.

**11 tests** :

```
test health_returns_ok             ... ok
test list_products_empty           ... ok
test list_products_returns_all     ... ok
test create_product_valid          ... ok
test create_product_name_empty     ... ok
test create_product_price_negative ... ok
test create_product_missing_body   ... ok
test get_product_found             ... ok
test get_product_not_found         ... ok
test update_product_partial        ... ok
test delete_product_then_not_found ... ok
```

---

## Points techniques notables

**`:id` vs `{id}`** — Axum 0.7 utilise `matchit 0.7` : la syntaxe des paramètres de route est `:id`. La syntaxe `{id}` (accolades) n'existe qu'à partir d'Axum 0.8 / matchit 0.8.

**`ValidatedJson<T>`** — Extractor custom qui chaîne désérialisation JSON et validation `validator` en une seule extraction. Tout handler qui l'utilise reçoit un payload garanti valide.

**`tokio::try_join!`** dans `list_products` — Les requêtes `fetch_all` et `COUNT(*)` sont lancées en parallèle sur le pool, réduisant la latence de la liste paginée.

**`Router<AppState>` typé explicitement** — Nécessaire avant `with_state()` pour que le compilateur vérifie que tous les handlers peuvent extraire `State<AppState>`.
