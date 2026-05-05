CREATE TABLE sac_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sac_id UUID NOT NULL REFERENCES sacs(id) ON DELETE CASCADE,
    produit_id UUID NOT NULL REFERENCES produits(id),
    quantite INTEGER NOT NULL CHECK (quantite > 0),
    ajoute_le TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sac_id, produit_id)
);
