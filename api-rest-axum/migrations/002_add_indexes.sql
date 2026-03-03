-- Migration 002 : ajout des index de performance
-- Règle : on n'indexe que ce qu'on interroge réellement dans les queries.

-- Index sur name : pour les recherches textuelles futures (LIKE, full-text)
-- Pas utile pour la pagination cursor-based sur id (qui utilise l'index PK),
-- mais indispensable si on ajoute un endpoint GET /products?search=...
CREATE INDEX IF NOT EXISTS idx_products_name ON products (name);

-- Index sur created_at : pour les tris chronologiques et les filtres de date
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at DESC);

-- Index sur price : pour les filtres de prix (range queries)
CREATE INDEX IF NOT EXISTS idx_products_price ON products (price);
