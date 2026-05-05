# PLAN.md — CityLunch API (Rust + Axum + Docker)

## Stack technique

| Composant        | Technologie                          |
|------------------|--------------------------------------|
| Langage          | Rust (edition 2021)                  |
| Framework HTTP   | Axum                                 |
| Base de données  | PostgreSQL 16                        |
| ORM / Query      | SQLx (async, compile-time checked)   |
| Auth JWT         | jsonwebtoken                         |
| Hachage mdp      | bcrypt (bcrypt crate)                |
| Email            | Lettre (SMTP)                        |
| Validation       | validator                            |
| Sérialisation    | serde / serde_json                   |
| Containerisation | Docker + Docker Compose              |
| Doc API          | utoipa + Swagger UI                  |
| Tests            | Rust built-in (`#[cfg(test)]`)       |

---

## Modèle Conceptuel de Données

```
┌─────────────────┐       ┌──────────────────┐       ┌─────────────────┐
│    produits     │       │   sac_items      │       │      sacs       │
├─────────────────┤       ├──────────────────┤       ├─────────────────┤
│ id (UUID)       │◄──────│ produit_id (FK)  │       │ id (UUID)       │
│ nom             │       │ sac_id (FK)      │──────►│ livreur_id (FK) │
│ description     │       │ quantite         │       │ created_at      │
│ prix (Decimal)  │       │ ajoute_le        │       └────────┬────────┘
│ type_produit    │       └──────────────────┘                │
│ disponible      │                                           │
│ created_at      │       ┌────────────────────┐              │
│ updated_at      │       │ mouvements_stock   │              │
└─────────────────┘       ├────────────────────┤              ▼
                          │ id (UUID)          │    ┌──────────────────┐
                          │ livreur_id (FK)    │◄───│    livreurs      │
                          │ produit_id (FK)    │    ├──────────────────┤
                          │ quantite           │    │ id (UUID)        │
                          │ type_mouvement     │    │ nom              │
                          │ created_at         │    │ prenom           │
                          └────────────────────┘    │ email (unique)   │
                                                    │ mot_de_passe     │
                                                    │ disponible       │
                                                    │ position_lat     │
                                                    │ position_lng     │
                                                    │ position_at      │
                                                    │ created_at       │
                                                    │ updated_at       │
                                                    └──────────────────┘
```

### Énumérations

- `type_produit` : `plat` | `dessert`
- `type_mouvement` : `chargement` | `retrait` | `retour_fin_service`

### Règles métier à retenir

1. Un sac appartient à un livreur. Il est unique et actif tant que le livreur est en service.
2. On ne peut pas mettre plus de produits dans le sac qu'il n'en existe en stock disponible.
3. Le retrait d'un produit du sac ne peut pas amener la quantité en dessous de 0.
4. Tous les mouvements de stock (chargement, retrait, retour) sont historisés dans `mouvements_stock`.
5. Un livreur créé par le gérant reçoit automatiquement un email avec son mot de passe temporaire.

---

## Structure du projet

```
citylunch/
├── docker-compose.yml
├── Dockerfile
├── .env.example
├── Cargo.toml
├── migrations/
│   ├── 20240001_create_produits.sql
│   ├── 20240002_create_livreurs.sql
│   ├── 20240003_create_sacs.sql
│   ├── 20240004_create_sac_items.sql
│   └── 20240005_create_mouvements_stock.sql
├── src/
│   ├── main.rs              # Bootstrap : DB pool, routes, Axum app
│   ├── config.rs            # Lecture des variables d'environnement
│   ├── db.rs                # Initialisation du pool SQLx
│   ├── errors.rs            # AppError -> réponse JSON + code HTTP
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── jwt.rs           # Génération & validation du token JWT
│   │   └── middleware.rs    # Extracteur Axum pour les routes protégées
│   ├── email/
│   │   └── mod.rs           # Envoi d'email via Lettre
│   ├── models/
│   │   ├── produit.rs
│   │   ├── livreur.rs
│   │   ├── sac.rs
│   │   └── mouvement_stock.rs
│   ├── handlers/
│   │   ├── produit.rs       # Handlers CRUD produits
│   │   ├── livreur.rs       # Handlers CRUD livreurs
│   │   ├── auth.rs          # Handler login JWT
│   │   └── sac.rs           # Handlers sac (protégés JWT)
│   └── routes/
│       └── mod.rs           # Déclaration et assemblage de toutes les routes
└── tests/
    └── business_rules.rs    # Test unitaire règle métier
```

---

## Étapes d'implémentation

### Étape 0 — Initialisation du projet

- [ ] `cargo new citylunch --edition 2021`
- [ ] Ajouter les dépendances dans `Cargo.toml` :
  ```toml
  axum = { version = "0.7", features = ["macros"] }
  tokio = { version = "1", features = ["full"] }
  sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono", "runtime-tokio-native-tls", "migrate"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  uuid = { version = "1", features = ["v4", "serde"] }
  chrono = { version = "0.4", features = ["serde"] }
  jsonwebtoken = "9"
  bcrypt = "0.15"
  lettre = { version = "0.11", features = ["tokio1", "tokio1-native-tls"] }
  validator = { version = "0.18", features = ["derive"] }
  dotenvy = "0.15"
  tower-http = { version = "0.5", features = ["cors", "trace"] }
  tracing = "0.1"
  tracing-subscriber = "0.3"
  thiserror = "1"
  utoipa = { version = "4", features = ["axum_extras", "uuid", "chrono"] }
  utoipa-swagger-ui = { version = "6", features = ["axum"] }
  ```
- [ ] Configurer `.env` (voir variables ci-dessous)
- [ ] `git init` + premier commit

### Étape 1 — Docker & infrastructure

- [ ] Écrire `docker-compose.yml` :
  ```yaml
  services:
    db:
      image: postgres:16
      environment:
        POSTGRES_DB: citylunch
        POSTGRES_USER: citylunch
        POSTGRES_PASSWORD: secret
      ports:
        - "5432:5432"
      volumes:
        - pg_data:/var/lib/postgresql/data

    mailhog:          # Serveur SMTP de dev pour les emails
      image: mailhog/mailhog
      ports:
        - "1025:1025"   # SMTP
        - "8025:8025"   # UI web

    api:
      build: .
      depends_on: [db, mailhog]
      env_file: .env
      ports:
        - "3000:3000"

  volumes:
    pg_data:
  ```
- [ ] Écrire le `Dockerfile` multi-stage (builder + image finale distroless/debian-slim)
- [ ] Fichier `.env.example` avec toutes les variables nécessaires :
  ```
  DATABASE_URL=postgres://citylunch:secret@db:5432/citylunch
  JWT_SECRET=change_me_in_production
  JWT_EXPIRATION_HOURS=24
  SMTP_HOST=mailhog
  SMTP_PORT=1025
  SMTP_FROM=noreply@citylunch.fr
  SERVER_PORT=3000
  ```

### Étape 2 — Migrations SQL

- [ ] Migration `produits` :
  ```sql
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
  ```
- [ ] Migration `livreurs` :
  ```sql
  CREATE TABLE livreurs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nom VARCHAR(255) NOT NULL,
    prenom VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    mot_de_passe VARCHAR(255) NOT NULL,
    disponible BOOLEAN NOT NULL DEFAULT true,
    position_lat DOUBLE PRECISION,
    position_lng DOUBLE PRECISION,
    position_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Migration `sacs` :
  ```sql
  CREATE TABLE sacs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    livreur_id UUID NOT NULL REFERENCES livreurs(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(livreur_id)    -- un seul sac actif par livreur
  );
  ```
- [ ] Migration `sac_items` :
  ```sql
  CREATE TABLE sac_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sac_id UUID NOT NULL REFERENCES sacs(id) ON DELETE CASCADE,
    produit_id UUID NOT NULL REFERENCES produits(id),
    quantite INTEGER NOT NULL CHECK (quantite > 0),
    ajoute_le TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sac_id, produit_id)
  );
  ```
- [ ] Migration `mouvements_stock` :
  ```sql
  CREATE TYPE type_mouvement AS ENUM ('chargement', 'retrait', 'retour_fin_service');
  CREATE TABLE mouvements_stock (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    livreur_id UUID NOT NULL REFERENCES livreurs(id),
    produit_id UUID NOT NULL REFERENCES produits(id),
    quantite INTEGER NOT NULL,
    type_mouvement type_mouvement NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Lancer les migrations avec `sqlx migrate run`

### Étape 3 — Gestion des erreurs (`errors.rs`)

- [ ] Définir `AppError` avec `thiserror` couvrant :
  - `NotFound` → 404
  - `Conflict` (ex: email déjà utilisé) → 409
  - `Unauthorized` → 401
  - `Forbidden` → 403
  - `BadRequest(String)` → 400
  - `InternalServerError` → 500
- [ ] Implémenter `IntoResponse` pour `AppError` → JSON `{ "error": "message" }`

### Étape 4 — Partie 1 : CRUD Produits

**Routes publiques :**

| Méthode | Route              | Description              |
|---------|--------------------|--------------------------|
| GET     | `/api/v1/produits` | Lister tous les produits |
| POST    | `/api/v1/produits` | Créer un produit         |
| GET     | `/api/v1/produits/:id` | Consulter un produit |
| PUT     | `/api/v1/produits/:id` | Modifier un produit  |
| DELETE  | `/api/v1/produits/:id` | Supprimer un produit |

**Modèles (`models/produit.rs`) :**
- `Produit` (lecture depuis BDD)
- `CreateProduitDto` (body de création, validé)
- `UpdateProduitDto` (body de modification, tous champs optionnels)

**Contraintes métier :**
- `prix` doit être >= 0
- `nom` obligatoire et non vide
- `type_produit` doit être `plat` ou `dessert`
- Suppression interdite si le produit est présent dans un sac actif → 409 Conflict

### Étape 5 — Partie 1 : CRUD Livreurs

**Routes publiques :**

| Méthode | Route               | Description               |
|---------|---------------------|---------------------------|
| GET     | `/api/v1/livreurs`  | Lister tous les livreurs  |
| POST    | `/api/v1/livreurs`  | Créer un livreur          |
| GET     | `/api/v1/livreurs/:id` | Consulter un livreur   |
| PUT     | `/api/v1/livreurs/:id` | Modifier un livreur    |
| DELETE  | `/api/v1/livreurs/:id` | Supprimer un livreur   |

**Contraintes métier :**
- `email` doit être unique → 409 Conflict si doublon
- `email` doit avoir un format valide
- Le mot de passe n'est **jamais** renvoyé dans les réponses JSON
- À la création : générer un mot de passe aléatoire (12 chars), le hacher avec bcrypt, envoyer l'original par email

**À la création d'un livreur :**
1. Générer un mot de passe aléatoire alphanumérique (12 caractères)
2. Hacher avec bcrypt (cost 12)
3. Sauvegarder le hash en base
4. Envoyer un email au livreur avec son email + mot de passe en clair
5. Créer automatiquement un `sac` vide pour ce livreur
6. Retourner le livreur créé (sans le mot de passe)

### Étape 6 — Partie 2 : Authentification JWT

**Route publique :**

| Méthode | Route                  | Description                        |
|---------|------------------------|------------------------------------|
| POST    | `/api/v1/auth/login`   | Login livreur → retourne JWT       |

**Logique :**
1. Récupérer le livreur par email
2. Comparer le mot de passe fourni avec le hash bcrypt
3. Générer un JWT signé avec `JWT_SECRET` contenant : `sub` (livreur_id), `exp`, `iat`
4. Retourner `{ "token": "..." }`

**Middleware JWT (`auth/middleware.rs`) :**
- Extracteur Axum `AuthenticatedLivreur` qui lit le header `Authorization: Bearer <token>`
- Vérifie la signature et l'expiration
- Injecte le `livreur_id` dans les handlers protégés

### Étape 7 — Partie 2 : Gestion du sac (routes protégées)

**Routes protégées (JWT requis) :**

| Méthode | Route                            | Description                    |
|---------|----------------------------------|--------------------------------|
| GET     | `/api/v1/sac`                    | Voir le contenu de son sac     |
| POST    | `/api/v1/sac/produits`           | Ajouter un produit dans le sac |
| DELETE  | `/api/v1/sac/produits/:produit_id` | Retirer un produit du sac    |

**Body `POST /api/v1/sac/produits` :**
```json
{ "produit_id": "uuid", "quantite": 3 }
```

**Contraintes métier :**
- Seul le livreur authentifié accède à **son propre sac**
- Le produit doit exister et être `disponible = true`
- La quantité ajoutée doit être > 0
- Si le produit est déjà dans le sac, **incrémenter** la quantité (pas dupliquer)
- On ne peut pas retirer plus de produits qu'il n'y en a dans le sac → 400
- Chaque opération crée une entrée dans `mouvements_stock`
  - Ajout → `type_mouvement = 'chargement'`
  - Retrait → `type_mouvement = 'retrait'`

**Réponse `GET /api/v1/sac` :**
```json
{
  "sac_id": "uuid",
  "livreur_id": "uuid",
  "items": [
    {
      "produit_id": "uuid",
      "nom": "Poulet rôti",
      "type_produit": "plat",
      "quantite": 2,
      "prix_unitaire": 12.50
    }
  ],
  "total_items": 2
}
```

### Étape 8 — Partie 3 : Test unitaire

**Règle métier testée : "On ne peut pas retirer plus de produits du sac qu'il n'en contient"**

Fichier : `tests/business_rules.rs` (ou `src/handlers/sac.rs` dans `#[cfg(test)]`)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn retrait_superieur_au_stock_est_invalide() {
        // Arrange : quantite_dans_sac = 2, quantite_retrait = 5
        // Act : valider_retrait(2, 5)
        // Assert : Err(AppError::BadRequest(...))
    }

    #[test]
    fn retrait_egal_au_stock_est_valide() {
        // quantite_dans_sac = 3, quantite_retrait = 3 → Ok
    }

    #[test]
    fn retrait_inferieur_au_stock_est_valide() {
        // quantite_dans_sac = 5, quantite_retrait = 2 → Ok
    }
}
```

La logique est extraite dans une fonction pure `valider_retrait(quantite_sac: i32, quantite_retrait: i32) -> Result<(), AppError>` facilement testable sans base de données.

---

## Routes API — Récapitulatif

### Publiques

| Méthode | Route                     | Status succès |
|---------|---------------------------|---------------|
| GET     | `/api/v1/produits`        | 200           |
| POST    | `/api/v1/produits`        | 201           |
| GET     | `/api/v1/produits/:id`    | 200           |
| PUT     | `/api/v1/produits/:id`    | 200           |
| DELETE  | `/api/v1/produits/:id`    | 204           |
| GET     | `/api/v1/livreurs`        | 200           |
| POST    | `/api/v1/livreurs`        | 201           |
| GET     | `/api/v1/livreurs/:id`    | 200           |
| PUT     | `/api/v1/livreurs/:id`    | 200           |
| DELETE  | `/api/v1/livreurs/:id`    | 204           |
| POST    | `/api/v1/auth/login`      | 200           |

### Protégées (JWT)

| Méthode | Route                               | Status succès |
|---------|-------------------------------------|---------------|
| GET     | `/api/v1/sac`                       | 200           |
| POST    | `/api/v1/sac/produits`              | 201           |
| DELETE  | `/api/v1/sac/produits/:produit_id`  | 204           |

### Documentation

| Route             | Description               |
|-------------------|---------------------------|
| `/swagger-ui`     | Interface Swagger UI       |
| `/api-docs/openapi.json` | Spec OpenAPI JSON   |

---

## Dockerfile (multi-stage)

```dockerfile
# Stage 1 : Build
FROM rust:1.76 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY ../../IdeaProject/citylunch/src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2 : Image finale
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/citylunch /usr/local/bin/citylunch
COPY migrations ./migrations
EXPOSE 3000
CMD ["citylunch"]
```

---

## Workflow Git

### Stratégie de branches

```
main        ──────────────────────────────────────────────► production
              ▲                                         ▲
              │ PR finale (fin de projet)               │
dev         ──┴──────────────────────────────────────────► intégration
              ▲     ▲     ▲     ▲     ▲     ▲     ▲
              │     │     │     │     │     │     │
feat/...  ────┘   feat/  feat/ feat/ feat/ feat/ feat/
```

| Branche              | Rôle                                                         | Direct push |
|----------------------|--------------------------------------------------------------|-------------|
| `main`               | Code livrable, toujours stable                               | Interdit    |
| `dev`                | Branche d'intégration, cible de toutes les PR de feature     | Interdit    |
| `feat/<nom>`         | Une feature = une branche, créée depuis `dev`                | Oui         |

### Nommage des branches de feature

| Étape                   | Nom de branche                  |
|-------------------------|---------------------------------|
| Étape 0 — Init projet   | `feat/setup-projet`             |
| Étape 1 — Docker        | `feat/docker-infrastructure`    |
| Étape 2 — Migrations    | `feat/migrations-sql`           |
| Étape 3 — Errors        | `feat/error-handling`           |
| Étape 4 — CRUD Produits | `feat/crud-produits`            |
| Étape 5 — CRUD Livreurs | `feat/crud-livreurs`            |
| Étape 6 — Auth JWT      | `feat/auth-jwt`                 |
| Étape 7 — Sac           | `feat/gestion-sac`              |
| Étape 8 — Tests         | `feat/tests-unitaires`          |
| OpenAPI / Swagger       | `feat/openapi-swagger`          |
| README                  | `feat/readme`                   |

### Convention de commits (Conventional Commits)

Format : `<type>(<scope>): <description courte en français>`

| Type       | Quand l'utiliser                                  | Exemple                                           |
|------------|---------------------------------------------------|---------------------------------------------------|
| `feat`     | Nouvelle fonctionnalité                           | `feat(produits): ajouter le handler POST`         |
| `fix`      | Correction de bug                                 | `fix(sac): corriger le check quantite negative`   |
| `chore`    | Config, tooling, dépendances                      | `chore: initialiser le projet Cargo`              |
| `docs`     | Documentation uniquement                          | `docs: ajouter README avec commandes d'init`      |
| `test`     | Ajout ou modification de tests                    | `test(sac): tester la règle retrait invalide`     |
| `refactor` | Refactoring sans changement de comportement       | `refactor(errors): extraire AppError dans module` |
| `ci`       | Configuration Docker, CI                          | `ci: ajouter docker-compose.yml`                  |

Règles :
- Un commit = une unité logique (pas de "fix stuff" fourre-tout)
- Description en minuscules, sans point final, en français
- Corps du commit optionnel pour expliquer le "pourquoi" si non évident

### Procédure complète par feature (à répéter pour chaque étape)

```bash
# 1. Se placer sur dev à jour
git checkout dev
git pull origin dev

# 2. Créer la branche de feature
git checkout -b feat/<nom-branche>

# 3. Développer — committer au fil de l'eau
git add <fichiers concernés>
git commit -m "feat(<scope>): <description>"
# ... répéter autant que nécessaire

# 4. Pousser la branche sur GitHub
git push -u origin feat/<nom-branche>

# 5. Ouvrir la Pull Request vers dev
gh pr create \
  --base dev \
  --head feat/<nom-branche> \
  --title "<type>(<scope>): <description>" \
  --body "$(cat <<'EOF'
## Résumé
- <bullet point des changements>

## Règles métier impactées
- <si applicable>

## Test
- [ ] `cargo build` passe sans erreur
- [ ] `cargo test` passe
EOF
)"

# 6. Merger la PR (sans supprimer la branche pour garder l'historique)
gh pr merge --squash --delete-branch

# 7. Revenir sur dev et tirer les changements
git checkout dev
git pull origin dev
```

### Initialisation du dépôt GitHub (à faire une seule fois, en tout premier)

```bash
# Depuis le répertoire racine du projet
git init
git add .
git commit -m "chore: initialisation du dépôt CityLunch"

# Créer le dépôt public sur GitHub
gh repo create citylunch --public --source=. --remote=origin --push

# Créer la branche dev depuis main
git checkout -b dev
git push -u origin dev

# Protéger main et dev contre les push directs (optionnel mais recommandé)
gh api repos/:owner/citylunch/branches/main/protection \
  --method PUT \
  --field required_pull_request_reviews[required_approving_review_count]=0 \
  --field enforce_admins=false \
  --field restrictions=null \
  --field required_status_checks=null 2>/dev/null || true
```

### PR finale dev → main (en toute fin de projet)

```bash
# Vérifier que dev est propre et tous les tests passent
git checkout dev
git pull origin dev
cargo test

# Ouvrir la PR de livraison
gh pr create \
  --base main \
  --head dev \
  --title "feat: implémentation complète CityLunch API" \
  --body "$(cat <<'EOF'
## Résumé

Implémentation complète de l'API CityLunch (Rust + Axum + Docker).

## Fonctionnalités livrées

- **Partie 1** : CRUD Produits et Livreurs (routes publiques)
- **Partie 2** : Authentification JWT, notification email à la création livreur, gestion du sac avec traçabilité des mouvements de stock
- **Partie 3** : Test unitaire de la règle métier de retrait du sac

## Lancer le projet

```bash
cp .env.example .env
docker compose up -d
```

## Lancer les tests

```bash
cargo test
```
EOF
)"

gh pr merge --merge
```

---

## Ordre de développement recommandé (avec Git intégré)

Chaque étape suit la procédure : **créer branche → développer → committer → PR → merge vers dev**

### 0. Init projet et dépôt GitHub
- Créer le dépôt GitHub public via `gh repo create`
- `cargo new citylunch --edition 2021`
- Ajouter `Cargo.toml` avec toutes les dépendances
- Ajouter `.env.example`, `.gitignore` (exclure `.env`, `target/`)
- **Branche** : `feat/setup-projet`
- **Commits** :
  - `chore: initialiser le projet Cargo avec les dépendances`
  - `chore: ajouter .gitignore et .env.example`
- **PR** → `dev`

### 1. Docker & infrastructure
- Écrire `docker-compose.yml` (PostgreSQL + MailHog + API)
- Écrire `Dockerfile` multi-stage
- **Branche** : `feat/docker-infrastructure`
- **Commits** :
  - `ci: ajouter docker-compose.yml avec PostgreSQL et MailHog`
  - `ci: ajouter Dockerfile multi-stage Rust`
- **PR** → `dev`

### 2. Migrations SQL
- Écrire les 5 fichiers de migration dans `migrations/`
- Vérifier avec `sqlx migrate run` (base Docker active)
- **Branche** : `feat/migrations-sql`
- **Commits** :
  - `chore(db): migration création table produits`
  - `chore(db): migration création table livreurs`
  - `chore(db): migration création tables sacs et sac_items`
  - `chore(db): migration création table mouvements_stock`
- **PR** → `dev`

### 3. Gestion des erreurs
- Implémenter `src/config.rs`, `src/db.rs`, `src/errors.rs`
- `main.rs` minimal qui boot l'app
- **Branche** : `feat/error-handling`
- **Commits** :
  - `chore: ajouter config.rs et db.rs avec pool SQLx`
  - `feat(errors): implémenter AppError avec IntoResponse JSON`
  - `feat: bootstrap main.rs avec routeur Axum`
- **PR** → `dev`

### 4. CRUD Produits
- `src/models/produit.rs`, `src/handlers/produit.rs`, routes
- **Branche** : `feat/crud-produits`
- **Commits** :
  - `feat(produits): ajouter modèles Produit et DTOs`
  - `feat(produits): implémenter handler GET liste et GET par id`
  - `feat(produits): implémenter handler POST avec validation`
  - `feat(produits): implémenter handler PUT`
  - `feat(produits): implémenter handler DELETE avec garde sac actif`
  - `feat(produits): brancher les routes dans le routeur`
- **PR** → `dev`

### 5. CRUD Livreurs + email
- `src/models/livreur.rs`, `src/email/mod.rs`, `src/handlers/livreur.rs`
- **Branche** : `feat/crud-livreurs`
- **Commits** :
  - `feat(livreurs): ajouter modèles Livreur et DTOs`
  - `feat(livreurs): implémenter handler GET liste et GET par id`
  - `feat(livreurs): implémenter handler POST avec génération mdp et hash bcrypt`
  - `feat(email): implémenter envoi email via Lettre + MailHog`
  - `feat(livreurs): brancher email et création sac automatique au POST`
  - `feat(livreurs): implémenter handler PUT et DELETE`
- **PR** → `dev`

### 6. Authentification JWT
- `src/auth/jwt.rs`, `src/auth/middleware.rs`, `src/handlers/auth.rs`
- **Branche** : `feat/auth-jwt`
- **Commits** :
  - `feat(auth): implémenter génération et validation JWT`
  - `feat(auth): implémenter extracteur AuthenticatedLivreur (middleware)`
  - `feat(auth): implémenter handler POST /auth/login`
  - `feat(auth): brancher la route login dans le routeur`
- **PR** → `dev`

### 7. Gestion du sac
- `src/models/sac.rs`, `src/models/mouvement_stock.rs`, `src/handlers/sac.rs`
- **Branche** : `feat/gestion-sac`
- **Commits** :
  - `feat(sac): ajouter modèles Sac, SacItem, MouvementStock`
  - `feat(sac): implémenter handler GET /sac avec jointure produits`
  - `feat(sac): implémenter handler POST /sac/produits avec règles métier`
  - `feat(sac): implémenter handler DELETE /sac/produits/:id`
  - `feat(sac): enregistrer mouvement_stock à chaque opération`
  - `feat(sac): brancher les routes protégées JWT`
- **PR** → `dev`

### 8. Tests unitaires
- Extraire `valider_retrait` en fonction pure
- Écrire les 3 cas de test dans `tests/business_rules.rs`
- **Branche** : `feat/tests-unitaires`
- **Commits** :
  - `refactor(sac): extraire valider_retrait en fonction pure testable`
  - `test(sac): tester retrait invalide supérieur au stock`
  - `test(sac): tester retrait valide égal et inférieur au stock`
- **PR** → `dev`

### 9. OpenAPI / Swagger
- Annoter les structs et handlers avec `utoipa`
- Exposer `/swagger-ui` et `/api-docs/openapi.json`
- **Branche** : `feat/openapi-swagger`
- **Commits** :
  - `docs(openapi): annoter les modèles Produit et Livreur`
  - `docs(openapi): annoter les handlers CRUD`
  - `docs(openapi): annoter les handlers auth et sac`
  - `docs(openapi): exposer Swagger UI et spec JSON`
- **PR** → `dev`

### 10. README
- Rédiger `README.md` complet
- **Branche** : `feat/readme`
- **Commits** :
  - `docs: rédiger README avec prérequis et commandes d'init`
  - `docs: ajouter MCD et description des routes dans README`
- **PR** → `dev`

### 11. PR finale dev → main
- Vérifier `cargo test` au vert
- Ouvrir la PR `dev` → `main` et merger

---

## Livrables finaux

- [ ] Code source complet sur GitHub (dépôt public)
- [ ] `README.md` avec prérequis, commandes d'init, commande de test
- [ ] Modèle conceptuel de données (section dans ce PLAN ou schéma PNG)
- [ ] Export OpenAPI (`/api-docs/openapi.json`) ou collection Bruno/Postman
- [ ] Au moins 1 test unitaire exécutable via `cargo test`
