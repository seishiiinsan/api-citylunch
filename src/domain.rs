/// Règles métier pures — sans dépendance à la base de données ni au réseau.
/// Chaque fonction retourne Result<(), String> pour rester indépendante d'AppError
/// et permettre des tests unitaires sans initialiser l'application.

// ─── Sac ───────────────────────────────────────────────────────────────────

/// Règle : on ne peut pas retirer plus de produits du sac qu'il n'en contient.
pub fn valider_retrait(quantite_sac: i32, quantite_retrait: i32) -> Result<(), String> {
    if quantite_retrait <= 0 {
        return Err(format!(
            "La quantité à retirer doit être > 0, reçu : {}",
            quantite_retrait
        ));
    }
    if quantite_retrait > quantite_sac {
        return Err(format!(
            "Impossible de retirer {} unité(s) : seulement {} disponible(s) dans le sac",
            quantite_retrait, quantite_sac
        ));
    }
    Ok(())
}

/// Règle : la quantité ajoutée au sac doit être strictement positive.
pub fn valider_quantite_ajout(quantite: i32) -> Result<(), String> {
    if quantite <= 0 {
        Err(format!(
            "La quantité à ajouter doit être > 0, reçu : {}",
            quantite
        ))
    } else {
        Ok(())
    }
}

// ─── Produit ───────────────────────────────────────────────────────────────

/// Règle : le prix d'un produit doit être >= 0.
pub fn valider_prix(prix: f64) -> Result<(), String> {
    if prix < 0.0 {
        Err(format!("Le prix doit être >= 0, reçu : {}", prix))
    } else {
        Ok(())
    }
}

/// Règle : le nom d'un produit ne peut pas être vide et ne dépasse pas 255 caractères.
pub fn valider_nom_produit(nom: &str) -> Result<(), String> {
    let trimmed = nom.trim();
    if trimmed.is_empty() {
        return Err("Le nom du produit ne peut pas être vide".to_string());
    }
    if trimmed.len() > 255 {
        return Err(format!(
            "Le nom du produit ne peut pas dépasser 255 caractères (reçu : {})",
            trimmed.len()
        ));
    }
    Ok(())
}

// ─── Livreur ───────────────────────────────────────────────────────────────

/// Règle : les coordonnées GPS doivent être dans les plages géographiques valides.
pub fn valider_coordonnees(lat: f64, lng: f64) -> Result<(), String> {
    if !(-90.0..=90.0).contains(&lat) {
        return Err(format!(
            "Latitude invalide : {} (doit être entre -90 et +90)",
            lat
        ));
    }
    if !(-180.0..=180.0).contains(&lng) {
        return Err(format!(
            "Longitude invalide : {} (doit être entre -180 et +180)",
            lng
        ));
    }
    Ok(())
}

/// Règle : le mot de passe généré doit avoir une longueur minimale et ne contenir
/// que des caractères alphanumériques (pas d'ambiguïté visuelle en email).
pub fn valider_format_password(password: &str) -> Result<(), String> {
    if password.len() < 16 {
        return Err(format!(
            "Le mot de passe généré est trop court : {} caractères (minimum 16)",
            password.len()
        ));
    }
    if !password.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("Le mot de passe généré contient des caractères non alphanumériques".to_string());
    }
    Ok(())
}

// ─── Sécurité ──────────────────────────────────────────────────────────────

const WEAK_SECRETS: &[&str] = &[
    "change_me_in_production",
    "secret",
    "password",
    "jwt_secret",
    "changeme",
    "",
];

/// Règle : le secret JWT doit être suffisamment long et non prévisible.
pub fn valider_force_secret_jwt(secret: &str) -> Result<(), String> {
    if WEAK_SECRETS.contains(&secret) {
        return Err(format!(
            "Secret JWT interdit (valeur connue publiquement) : '{}'",
            secret
        ));
    }
    if secret.len() < 32 {
        return Err(format!(
            "Secret JWT trop court : {} caractères (minimum 32)",
            secret.len()
        ));
    }
    Ok(())
}
