-- Migration 001 : création de la table products
-- UUID comme PK : pas de séquence int à gérer, safe pour le sharding futur
-- timestamptz (avec timezone) plutôt que timestamp : recommandé en prod PostgreSQL

CREATE TABLE IF NOT EXISTS products (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT        NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    description TEXT,
    price       NUMERIC(10, 2) NOT NULL CHECK (price > 0),
    stock       INTEGER     NOT NULL DEFAULT 0 CHECK (stock >= 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Trigger pour mettre à jour updated_at automatiquement à chaque UPDATE
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER products_updated_at
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
