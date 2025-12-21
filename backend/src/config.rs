use std::env;

#[derive(Clone, Debug)]
pub struct Config {

    pub database_url: String,
    

    pub redis_url: String,
    

    pub google_client_id: String,
    pub google_client_secret: String,
    

    pub jwt_secret: String,
    

    pub encryption_key: String,
    

    pub frontend_url: String,
    pub base_url: String,
    pub backend_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let backend_port: u16 = env::var("BACKEND_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);
        
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}", backend_port));
        
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            google_client_id: env::var("GOOGLE_CLIENT_ID")?,
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")?,
            jwt_secret: env::var("JWT_SECRET")?,
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| env::var("JWT_SECRET").unwrap_or_else(|_| "default-key".to_string())),
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            base_url,
            backend_port,
        })
    }
}
