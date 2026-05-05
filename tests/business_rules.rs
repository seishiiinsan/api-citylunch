/// Tests unitaires des règles métier CityLunch
///
/// Chaque module correspond à une entité du domaine.
/// Aucun accès à la base de données — fonctions pures uniquement.
///
/// Lancer : cargo test
use api_citylunch::domain;

// ═══════════════════════════════════════════════════════════════════════════
// SAC — gestion des quantités
// ═══════════════════════════════════════════════════════════════════════════

mod sac {
    use super::*;

    // ── Règle : on ne peut pas retirer plus qu'il n'y en a dans le sac ──

    #[test]
    fn retrait_superieur_au_stock_est_invalide() {
        assert!(domain::valider_retrait(2, 5).is_err());
    }

    #[test]
    fn retrait_egal_au_stock_est_valide() {
        assert!(domain::valider_retrait(3, 3).is_ok());
    }

    #[test]
    fn retrait_inferieur_au_stock_est_valide() {
        assert!(domain::valider_retrait(5, 2).is_ok());
    }

    #[test]
    fn retrait_zero_est_invalide() {
        let err = domain::valider_retrait(5, 0).unwrap_err();
        assert!(err.contains("> 0"), "message attendu '>0', reçu : {}", err);
    }

    #[test]
    fn retrait_negatif_est_invalide() {
        assert!(domain::valider_retrait(5, -1).is_err());
    }

    #[test]
    fn retrait_depuis_sac_vide_est_invalide() {
        assert!(domain::valider_retrait(0, 1).is_err());
    }

    // ── Règle : la quantité ajoutée doit être strictement positive ──

    #[test]
    fn ajout_quantite_positive_est_valide() {
        assert!(domain::valider_quantite_ajout(1).is_ok());
        assert!(domain::valider_quantite_ajout(100).is_ok());
    }

    #[test]
    fn ajout_quantite_zero_est_invalide() {
        assert!(domain::valider_quantite_ajout(0).is_err());
    }

    #[test]
    fn ajout_quantite_negative_est_invalide() {
        assert!(domain::valider_quantite_ajout(-3).is_err());
    }

    // ── Message d'erreur explicite (aide au débogage client) ──

    #[test]
    fn message_retrait_contient_les_quantites() {
        let err = domain::valider_retrait(2, 5).unwrap_err();
        assert!(err.contains('5'), "doit mentionner la quantité retirée");
        assert!(err.contains('2'), "doit mentionner la quantité disponible");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PRODUIT — validation des attributs
// ═══════════════════════════════════════════════════════════════════════════

mod produit {
    use super::*;

    // ── Règle : le prix doit être >= 0 ──

    #[test]
    fn prix_positif_est_valide() {
        assert!(domain::valider_prix(0.0).is_ok());
        assert!(domain::valider_prix(12.50).is_ok());
        assert!(domain::valider_prix(999.99).is_ok());
    }

    #[test]
    fn prix_negatif_est_invalide() {
        assert!(domain::valider_prix(-0.01).is_err());
        assert!(domain::valider_prix(-100.0).is_err());
    }

    #[test]
    fn prix_zero_est_valide() {
        // Un produit offert (prix = 0) est autorisé
        assert!(domain::valider_prix(0.0).is_ok());
    }

    // ── Règle : le nom ne peut pas être vide et <= 255 caractères ──

    #[test]
    fn nom_valide_est_accepte() {
        assert!(domain::valider_nom_produit("Poulet rôti").is_ok());
        assert!(domain::valider_nom_produit("A").is_ok());
    }

    #[test]
    fn nom_vide_est_refuse() {
        assert!(domain::valider_nom_produit("").is_err());
    }

    #[test]
    fn nom_espaces_seuls_est_refuse() {
        assert!(domain::valider_nom_produit("   ").is_err());
    }

    #[test]
    fn nom_trop_long_est_refuse() {
        let nom_256 = "a".repeat(256);
        assert!(domain::valider_nom_produit(&nom_256).is_err());
    }

    #[test]
    fn nom_exactement_255_caracteres_est_valide() {
        let nom_255 = "a".repeat(255);
        assert!(domain::valider_nom_produit(&nom_255).is_ok());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LIVREUR — position GPS
// ═══════════════════════════════════════════════════════════════════════════

mod livreur {
    use super::*;

    // ── Règle : la position transmise doit être dans les plages géographiques ──

    #[test]
    fn coordonnees_paris_sont_valides() {
        // Paris : 48.8566° N, 2.3522° E
        assert!(domain::valider_coordonnees(48.8566, 2.3522).is_ok());
    }

    #[test]
    fn pole_nord_est_valide() {
        assert!(domain::valider_coordonnees(90.0, 0.0).is_ok());
    }

    #[test]
    fn pole_sud_est_valide() {
        assert!(domain::valider_coordonnees(-90.0, 0.0).is_ok());
    }

    #[test]
    fn latitude_superieure_a_90_est_invalide() {
        assert!(domain::valider_coordonnees(90.01, 0.0).is_err());
    }

    #[test]
    fn latitude_inferieure_a_moins_90_est_invalide() {
        assert!(domain::valider_coordonnees(-90.01, 0.0).is_err());
    }

    #[test]
    fn longitude_superieure_a_180_est_invalide() {
        assert!(domain::valider_coordonnees(0.0, 180.01).is_err());
    }

    #[test]
    fn longitude_inferieure_a_moins_180_est_invalide() {
        assert!(domain::valider_coordonnees(0.0, -180.01).is_err());
    }

    #[test]
    fn coordonnees_impossibles_999_sont_invalides() {
        // Valeur absurde que le sujet imposait de bloquer
        assert!(domain::valider_coordonnees(999.0, 999.0).is_err());
    }

    // ── Règle : le mot de passe généré doit être sûr ──

    #[test]
    fn password_genere_a_longueur_suffisante() {
        assert!(domain::valider_format_password("aB3dEf4gHiJkLmNo").is_ok());
    }

    #[test]
    fn password_trop_court_est_refuse() {
        assert!(domain::valider_format_password("abc123").is_err());
    }

    #[test]
    fn password_avec_caracteres_speciaux_est_refuse() {
        // Le générateur produit uniquement des alphanumériques
        // pour éviter les problèmes d'affichage dans les emails
        assert!(domain::valider_format_password("aB3dEf4gHiJkLm!@").is_err());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SÉCURITÉ — force du secret JWT
// ═══════════════════════════════════════════════════════════════════════════

mod securite {
    use super::*;

    // ── Règle : le secret JWT doit être long et non prévisible ──

    #[test]
    fn secret_32_caracteres_aleatoires_est_valide() {
        assert!(domain::valider_force_secret_jwt(
            "ab319d2b314c789a55d590fb368ce8ef"
        )
        .is_ok());
    }

    #[test]
    fn secret_64_caracteres_est_valide() {
        assert!(domain::valider_force_secret_jwt(
            "ab319d2b314c789a55d590fb368ce8ef0b576b0b4932644d1f9d21f2ecc3c6bd"
        )
        .is_ok());
    }

    #[test]
    fn secret_trop_court_est_refuse() {
        assert!(domain::valider_force_secret_jwt("courtsecret").is_err());
    }

    #[test]
    fn secret_vide_est_refuse() {
        assert!(domain::valider_force_secret_jwt("").is_err());
    }

    #[test]
    fn secret_change_me_in_production_est_refuse() {
        // Valeur par défaut du .env.example — doit être rejetée
        assert!(domain::valider_force_secret_jwt("change_me_in_production").is_err());
    }

    #[test]
    fn secret_password_est_refuse() {
        assert!(domain::valider_force_secret_jwt("password").is_err());
    }

    #[test]
    fn message_erreur_secret_court_mentionne_longueur() {
        let err = domain::valider_force_secret_jwt("court").unwrap_err();
        assert!(err.contains("32"), "doit mentionner le minimum de 32 chars");
    }
}
