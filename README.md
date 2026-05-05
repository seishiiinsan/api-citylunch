# CityLunch API

API REST pour la gestion des livreurs, des produits et des sacs CityLunch.  
Construite en **Rust** avec **Axum**, **SQLx** et **PostgreSQL**.

Dépôt GitHub : https://github.com/seishiiinsan/api-citylunch

---

## Sommaire

- [Prérequis](#prérequis)
- [Installation](#installation)
- [Configuration](#configuration)
- [Lancer le projet](#lancer-le-projet)
- [Lancer les tests](#lancer-les-tests)
- [Routes de l'API](#routes-de-lapi)
- [Modèle conceptuel de données](#modèle-conceptuel-de-données)
- [Tester les routes](#tester-les-routes)

---

## Prérequis

| Outil | Version minimale | Installation |
|---|---|---|
| Rust + Cargo | 1.75 | https://rustup.rs |
| Docker + Docker Compose | 24 | https://docs.docker.com/get-docker |
| Git | 2.x | https://git-scm.com |

> Docker est utilisé pour PostgreSQL (base de données) et Mailpit (serveur SMTP de développement).  
> L'API elle-même se lance directement avec `cargo run`.

---

## Installation

```bash
# 1. Cloner le dépôt
git clone https://github.com/seishiiinsan/api-citylunch.git
cd api-citylunch

# 2. Copier le fichier d'environnement
cp .env.example .env

# 3. Générer un secret JWT fort (remplacer la valeur dans .env)
openssl rand -hex 32
# → coller la valeur dans JWT_SECRET=... dans .env
```

---

## Configuration

Toutes les variables sont dans `.env` (copié depuis `.env.example`) :

| Variable | Description | Valeur par défaut |
|---|---|---|
| `DATABASE_URL` | URL de connexion PostgreSQL | `postgres://citylunch:secret@localhost:5432/citylunch` |
| `JWT_SECRET` | Secret de signature des tokens JWT (**minimum 32 caractères**) | _à générer_ |
| `JWT_EXPIRATION_HOURS` | Durée de validité des tokens en heures | `24` |
| `SMTP_HOST` | Hôte SMTP (Mailpit en dev) | `localhost` |
| `SMTP_PORT` | Port SMTP | `1025` |
| `SMTP_FROM` | Adresse expéditeur des emails | `noreply@citylunch.fr` |
| `SERVER_PORT` | Port d'écoute de l'API | `3000` |

> **Important** : l'API refuse de démarrer si `JWT_SECRET` est vide, trop court (< 32 caractères) ou correspond à une valeur connue faible (`secret`, `changeme`, etc.).

---

## Lancer le projet

```bash
# Démarrer PostgreSQL + Mailpit
docker compose up -d

# Lancer l'API (les migrations SQL s'appliquent automatiquement au démarrage)
cargo run
```

L'API est disponible sur **http://localhost:3000**.  
L'interface Mailpit (emails) est disponible sur **http://localhost:8026**.

### Vérifier que l'API fonctionne

```bash
curl http://localhost:3000/health
# → {"status":"ok"}
```

### Arrêter

```bash
# Arrêter l'API : Ctrl+C dans le terminal cargo run

# Arrêter les conteneurs Docker
docker compose down
```

---

## Lancer les tests

```bash
cargo test
```

Résultat attendu :

```
test result: ok. 49 passed; 0 failed
```

### Détail des tests

Les tests sont dans `tests/` et couvrent les règles métier sans base de données :

| Fichier | Règles testées | Nombre |
|---|---|---|
| `tests/business_rules.rs` | Retrait sac, prix produit, coordonnées GPS, mot de passe, secret JWT | 36 |
| `tests/jwt_tests.rs` | Token valide, expiré, mauvais secret, trafiqué | 7 |
| `tests/errors_tests.rs` | Codes HTTP 400/401/404/409/201/204 | 6 |

### Mesurer la couverture (optionnel)

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --lib --tests
```

---

## Routes de l'API

### Publiques

| Méthode | Route | Description | Status |
|---|---|---|---|
| `GET` | `/health` | Health check | 200 |
| `GET` | `/api/v1/produits` | Lister tous les produits | 200 |
| `POST` | `/api/v1/produits` | Créer un produit | 201 |
| `GET` | `/api/v1/produits/:id` | Consulter un produit | 200 |
| `PUT` | `/api/v1/produits/:id` | Modifier un produit | 200 |
| `DELETE` | `/api/v1/produits/:id` | Supprimer un produit | 204 |
| `GET` | `/api/v1/livreurs` | Lister tous les livreurs | 200 |
| `POST` | `/api/v1/livreurs` | Créer un livreur (envoie les credentials par email) | 201 |
| `GET` | `/api/v1/livreurs/:id` | Consulter un livreur | 200 |
| `PUT` | `/api/v1/livreurs/:id` | Modifier un livreur | 200 |
| `DELETE` | `/api/v1/livreurs/:id` | Supprimer un livreur | 204 |
| `GET` | `/api/v1/livreurs/:id/sac` | Voir le sac d'un livreur (vue gérant) | 200 |
| `GET` | `/api/v1/livreurs/:id/mouvements` | Historique des mouvements de stock | 200 |
| `POST` | `/api/v1/auth/login` | Connexion → token JWT | 200 |

### Protégées — JWT requis (`Authorization: Bearer <token>`)

| Méthode | Route | Description | Status |
|---|---|---|---|
| `GET` | `/api/v1/sac` | Voir son propre sac | 200 |
| `POST` | `/api/v1/sac/produits` | Ajouter un produit au sac | 201 |
| `DELETE` | `/api/v1/sac/produits/:id` | Retirer une quantité d'un produit du sac | 204 |
| `POST` | `/api/v1/sac/fin-service` | Retourner les produits non livrés (fin de service) | 200 |

### Codes d'erreur

| Code | Signification |
|---|---|
| `400` | Données invalides (validation échouée, quantité négative, prix < 0…) |
| `401` | Non authentifié ou token invalide/expiré |
| `404` | Ressource introuvable |
| `409` | Conflit (email déjà utilisé, produit présent dans un sac) |
| `500` | Erreur interne serveur |

Toutes les erreurs retournent `{ "error": "message explicatif" }`.

---

## Schémas des requêtes

### `POST /api/v1/produits`
```json
{
  "nom": "Poulet rôti aux herbes",
  "description": "Optionnel — max 2000 caractères",
  "prix": 12.50,
  "type_produit": "plat",
  "disponible": true
}
```
`type_produit` : `"plat"` ou `"dessert"`

### `PUT /api/v1/produits/:id`
Tous les champs sont optionnels — seuls les champs fournis sont mis à jour.

### `POST /api/v1/livreurs`
```json
{
  "nom": "Dupont",
  "prenom": "Jean",
  "email": "jean.dupont@citylunch.fr"
}
```
Un mot de passe aléatoire est généré et envoyé à l'adresse email.  
Un sac vide est automatiquement créé pour le livreur.

### `PUT /api/v1/livreurs/:id`
```json
{
  "nom": "Optionnel",
  "prenom": "Optionnel",
  "email": "optionnel@exemple.fr",
  "disponible": false,
  "position_lat": 48.8566,
  "position_lng": 2.3522
}
```
`position_lat` : entre -90 et +90 — `position_lng` : entre -180 et +180  
Quand lat/lng sont fournis, `position_at` est mis à jour automatiquement.

### `POST /api/v1/auth/login`
```json
{
  "email": "jean.dupont@citylunch.fr",
  "mot_de_passe": "motDePasseRecu"
}
```
Retourne `{ "token": "eyJ..." }`. Utiliser ce token dans le header `Authorization: Bearer <token>`.

### `POST /api/v1/sac/produits`
```json
{
  "produit_id": "uuid-du-produit",
  "quantite": 3
}
```
Si le produit est déjà dans le sac, la quantité est incrémentée.

### `DELETE /api/v1/sac/produits/:id`
```json
{
  "quantite": 2
}
```
Retrait partiel ou total. Erreur 400 si la quantité dépasse ce qui est dans le sac.

### `GET /api/v1/sac` — Exemple de réponse
```json
{
  "sac_id": "uuid",
  "livreur_id": "uuid",
  "items": [
    {
      "produit_id": "uuid",
      "nom": "Poulet rôti aux herbes",
      "type_produit": "plat",
      "quantite": 3,
      "prix_unitaire": "12.50"
    }
  ],
  "total_items": 3
}
```

---

## Modèle conceptuel de données

```
┌─────────────────┐       ┌──────────────────┐       ┌─────────────────┐
│    produits     │       │   sac_items      │       │      sacs       │
├─────────────────┤       ├──────────────────┤       ├─────────────────┤
│ id (UUID) PK    │◄──────│ produit_id (FK)  │       │ id (UUID) PK    │
│ nom             │       │ sac_id (FK)      │──────►│ livreur_id (FK) │
│ description     │       │ quantite         │       │ created_at      │
│ prix (Decimal)  │       │ ajoute_le        │       └────────┬────────┘
│ type_produit    │       └──────────────────┘                │
│ disponible      │                                           │
│ created_at      │       ┌────────────────────┐              │
│ updated_at      │       │ mouvements_stock   │              ▼
└─────────────────┘       ├────────────────────┤    ┌──────────────────┐
                          │ id (UUID) PK       │    │    livreurs      │
                          │ livreur_id (FK)    │◄───├──────────────────┤
                          │ produit_id (FK)    │    │ id (UUID) PK     │
                          │ quantite           │    │ nom              │
                          │ type_mouvement     │    │ prenom           │
                          │ created_at         │    │ email (unique)   │
                          └────────────────────┘    │ mot_de_passe     │
                                                    │ disponible       │
                                                    │ position_lat     │
                                                    │ position_lng     │
                                                    │ position_at      │
                                                    │ created_at       │
                                                    │ updated_at       │
                                                    └──────────────────┘
```

**Énumérations**
- `type_produit` : `plat` | `dessert`
- `type_mouvement` : `chargement` | `retrait` | `retour_fin_service`

**Contraintes métier**
- Un livreur a exactement un sac (UNIQUE sur `livreur_id`)
- Un produit ne peut être supprimé s'il est présent dans un sac
- Le retrait d'un produit du sac ne peut pas amener la quantité en dessous de 0
- Un produit doit être `disponible = true` pour être ajouté à un sac

---

## Tester les routes

Le fichier `citylunch.http` contient toutes les routes avec des données réelles.  
Il est compatible avec l'extension VS Code **REST Client** (`humao.rest-client`).

```bash
# Installer l'extension VS Code
code --install-extension humao.rest-client
```

Ouvrir `citylunch.http` dans VS Code → cliquer **Send Request** au-dessus de chaque bloc.

> Les tokens JWT dans `citylunch.http` expirent 24h après la génération du jeu de données.  
> Pour les régénérer : exécuter les blocs `POST /auth/login` et mettre à jour les variables `@token_*`.

La spec OpenAPI complète est disponible dans `openapi.yaml`.

Une **interface Swagger UI interactive** est accessible directement depuis l'API en cours d'exécution :

| URL | Description |
|---|---|
| `http://localhost:3000/swagger-ui` | Interface interactive — tester les routes depuis le navigateur |
| `http://localhost:3000/api-docs/openapi.json` | Spec OpenAPI 3.0 au format JSON |

> La Swagger UI permet de tester toutes les routes sans outil externe. Pour les routes protégées, cliquer **Authorize** en haut à droite et saisir le token JWT obtenu via `POST /api/v1/auth/login`.

---

## Structure du projet

```
api-citylunch/
├── src/
│   ├── main.rs              # Bootstrap : DB pool, migrations, routeur Axum
│   ├── lib.rs               # Entrée bibliothèque (expose domain pour les tests)
│   ├── config.rs            # Lecture des variables d'environnement
│   ├── db.rs                # Initialisation du pool SQLx
│   ├── domain.rs            # Règles métier pures (testables sans DB)
│   ├── errors.rs            # AppError → réponse JSON + code HTTP
│   ├── auth/
│   │   ├── jwt.rs           # Génération & validation du token JWT
│   │   └── middleware.rs    # Extracteur Axum AuthenticatedLivreur
│   ├── email/mod.rs         # Envoi d'email via Lettre (SMTP)
│   ├── models/              # Structs de données et DTOs
│   ├── handlers/            # Logique de chaque endpoint
│   └── routes/mod.rs        # Assemblage des routes + AppState
├── tests/
│   ├── business_rules.rs    # 36 tests des règles métier
│   ├── jwt_tests.rs         # 7 tests JWT
│   └── errors_tests.rs      # 6 tests codes HTTP
├── migrations/              # 5 fichiers SQL (sqlx migrate run)
├── citylunch.http           # Collection de routes pour VS Code REST Client
├── openapi.yaml             # Spec OpenAPI 3.0
├── docker-compose.yml       # PostgreSQL + Mailpit
├── Dockerfile               # Build multi-stage Rust
└── .env.example             # Variables d'environnement à copier
```
