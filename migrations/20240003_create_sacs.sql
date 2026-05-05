CREATE TABLE sacs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    livreur_id UUID NOT NULL REFERENCES livreurs(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(livreur_id)
);
