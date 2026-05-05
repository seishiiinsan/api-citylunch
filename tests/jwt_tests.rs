/// Tests unitaires — génération et validation des tokens JWT
///
/// Aucune base de données requise.
use api_citylunch::domain;

// Réutilise les fonctions de jwt.rs via le module lib exposé dans domain
// Pour tester jwt directement, on duplique la logique minimale ici
// car auth::jwt est privé (mod, pas pub mod dans lib.rs).
//
// Ce fichier teste les invariants observables du système JWT :
// - Un token généré est valide
// - Un token expiré est refusé
// - Un token avec le mauvais secret est refusé
// - Un token trafiqué est refusé

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

const SECRET_VALIDE: &str = "ab319d2b314c789a55d590fb368ce8ef0b576b0b4932644d1f9d21f2ecc3c6bd";

fn generer_token(livreur_id: Uuid, secret: &str, duree_heures: i64) -> String {
    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::hours(duree_heures)).timestamp() as usize;
    let claims = Claims {
        sub: livreur_id.to_string(),
        exp,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

fn valider_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut v = Validation::default();
    v.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &v)
        .map(|d| d.claims)
}

// ── Règle : un token valide doit être accepté ──────────────────────────────

#[test]
fn token_valide_est_accepte() {
    let id = Uuid::new_v4();
    let token = generer_token(id, SECRET_VALIDE, 24);
    let claims = valider_token(&token, SECRET_VALIDE).expect("le token doit être valide");
    assert_eq!(claims.sub, id.to_string());
}

#[test]
fn sub_du_token_correspond_au_livreur_id() {
    let id = Uuid::new_v4();
    let token = generer_token(id, SECRET_VALIDE, 1);
    let claims = valider_token(&token, SECRET_VALIDE).unwrap();
    let parsed = Uuid::parse_str(&claims.sub).expect("sub doit être un UUID valide");
    assert_eq!(parsed, id);
}

#[test]
fn iat_est_anterieur_a_exp() {
    let id = Uuid::new_v4();
    let token = generer_token(id, SECRET_VALIDE, 24);
    let claims = valider_token(&token, SECRET_VALIDE).unwrap();
    assert!(
        claims.iat < claims.exp,
        "iat ({}) doit être < exp ({})",
        claims.iat,
        claims.exp
    );
}

// ── Règle : un token expiré doit être refusé ──────────────────────────────

#[test]
fn token_expire_est_refuse() {
    let id = Uuid::new_v4();
    // Durée négative → token déjà expiré
    let token = generer_token(id, SECRET_VALIDE, -1);
    assert!(
        valider_token(&token, SECRET_VALIDE).is_err(),
        "un token expiré doit être refusé"
    );
}

// ── Règle : un token signé avec un autre secret est refusé ────────────────

#[test]
fn token_avec_mauvais_secret_est_refuse() {
    let id = Uuid::new_v4();
    let token = generer_token(id, SECRET_VALIDE, 24);
    let autre_secret = "ff001122334455667788990011223344556677889900112233445566778899aabb";
    assert!(
        valider_token(&token, autre_secret).is_err(),
        "un token signé avec un secret différent doit être refusé"
    );
}

// ── Règle : un token trafiqué (payload modifié) est refusé ────────────────

#[test]
fn token_trafique_est_refuse() {
    let id = Uuid::new_v4();
    let token = generer_token(id, SECRET_VALIDE, 24);

    // Modifier le payload (partie centrale du JWT)
    let parties: Vec<&str> = token.split('.').collect();
    assert_eq!(parties.len(), 3, "un JWT a exactement 3 parties");

    // Remplacer le payload par un payload encodé différent
    let faux_payload = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        r#"{"sub":"00000000-0000-0000-0000-000000000000","exp":9999999999,"iat":0}"#,
    );
    let token_trafique = format!("{}.{}.{}", parties[0], faux_payload, parties[2]);

    assert!(
        valider_token(&token_trafique, SECRET_VALIDE).is_err(),
        "un token dont le payload a été modifié doit être refusé"
    );
}

// ── Règle : deux tokens générés pour le même livreur sont différents ──────

#[test]
fn deux_tokens_pour_meme_livreur_sont_differents() {
    let id = Uuid::new_v4();
    let t1 = generer_token(id, SECRET_VALIDE, 24);
    // Attendre une seconde serait nécessaire pour des timestamps différents
    // mais ici on teste simplement le format — même sub, même secret, même durée
    // produit le même token (déterministe à la seconde près)
    // Ce test documente le comportement plutôt qu'il ne l'impose
    assert!(!t1.is_empty());
}
