CREATE TYPE type_produit AS ENUM ('plat', 'dessert');

CREATE TABLE produits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nom VARCHAR(255) NOT NULL,
    description TEXT,
    prix NUMERIC(10,2) NOT NULL CHECK (prix >= 0),
    type_produit type_produit NOT NULL,
    disponible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
