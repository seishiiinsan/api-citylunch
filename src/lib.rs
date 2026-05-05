/// Point d'entrée de la crate en tant que bibliothèque.
/// Seuls les modules sans dépendance infrastructure sont exposés.
/// Les handlers (SQLx, Axum) ne peuvent pas être testés sans DB active.
pub mod domain;
