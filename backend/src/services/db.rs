use sqlx::{postgres::PgPoolOptions, PgPool, migrate::Migrator};
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        let migrator = Migrator::new(Path::new("./migrations")).await?;
        migrator.run(&self.pool).await?;
        Ok(())
    }
}
