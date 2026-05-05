use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_from: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a number"),
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "1025".to_string())
                .parse()
                .expect("SMTP_PORT must be a number"),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@citylunch.fr".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
        }
    }
}
