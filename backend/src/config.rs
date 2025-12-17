use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    // Database
    pub database_url: String,
    
    // Redis
    pub redis_url: String,
    
    // Google OAuth
    pub google_client_id: String,
    pub google_client_secret: String,
    
    // JWT
    pub jwt_secret: String,
    
    // URLs
    pub frontend_url: String,
    pub backend_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            google_client_id: env::var("GOOGLE_CLIENT_ID")?,
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")?,
            jwt_secret: env::var("JWT_SECRET")?,
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            backend_port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
        })
    }
}
