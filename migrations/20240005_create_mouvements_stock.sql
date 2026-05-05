CREATE TYPE type_mouvement AS ENUM ('chargement', 'retrait', 'retour_fin_service');

CREATE TABLE mouvements_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    livreur_id UUID NOT NULL REFERENCES livreurs(id),
    produit_id UUID NOT NULL REFERENCES produits(id),
    quantite INTEGER NOT NULL,
    type_mouvement type_mouvement NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
