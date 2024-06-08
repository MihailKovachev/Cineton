pub mod db_error;

use db_error::DBError;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct DBConfig {
    db_name: String,
    db_host: String,
    db_port: u16,
    db_user: String,
    db_pass: String
}

pub fn parse_config() -> Result<DBConfig, Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string("db_config.json")?;
    
    let config: DBConfig = serde_json::from_str(&config)?;

    Ok(config)
}


pub async fn connect() -> Result<sqlx::Pool<sqlx::Postgres>, DBError> {
    
    let Ok(db_config) = parse_config() else { return Err(DBError::ConfigError(String::from("Failed to parse database config"))); };

    let url = format!("postgres://{}:{}@{}:{}/{}", db_config.db_user, db_config.db_pass, db_config.db_host, db_config.db_port, db_config.db_name);

    match sqlx::postgres::PgPool::connect(&url).await {
        Ok(pool) => {
            return Ok(pool)
        },
        Err(err) => {
            return Err(DBError::ConnectionError(err.to_string()));
        }
    }
}

