use lettre::{
    message::header::ContentType,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::{config::Config, errors::AppError};

pub async fn send_credentials_email(
    config: &Config,
    to_email: &str,
    to_name: &str,
    password: &str,
) -> Result<(), AppError> {
    let email = Message::builder()
        .from(
            config
                .smtp_from
                .parse()
                .map_err(|_| AppError::InternalServerError)?,
        )
        .to(format!("{} <{}>", to_name, to_email)
            .parse()
            .map_err(|_| AppError::InternalServerError)?)
        .subject("Vos identifiants CityLunch")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Bonjour {},\n\nVotre compte livreur CityLunch a été créé.\n\nEmail : {}\nMot de passe : {}\n\nMerci de changer votre mot de passe à la première connexion.\n\nL'équipe CityLunch",
            to_name, to_email, password
        ))
        .map_err(|_| AppError::InternalServerError)?;

    // En développement (Mailpit sur port 1025) : connexion non chiffrée acceptable.
    // En production : remplacer par starttls_relay() ou tls_relay() avec
    // des credentials SMTP et un serveur qui supporte TLS.
    // `builder_dangerous` est nommé ainsi intentionnellement par la bibliothèque
    // pour signaler l'absence de vérification TLS — à ne pas utiliser en production.
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.smtp_host)
            .port(config.smtp_port)
            .build();

    mailer.send(email).await.map_err(|e| {
        tracing::error!("Failed to send email: {:?}", e);
        AppError::InternalServerError
    })?;

    Ok(())
}
