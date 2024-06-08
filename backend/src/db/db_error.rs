use core::fmt;

#[derive(Debug, Clone)]
pub enum DBError {
    ConfigError(String),
    ConnectionError(String)
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Failed to parse database configuration file: {}", msg),
            Self::ConnectionError(msg) => write!(f, "Failed to establish connection with the database: {}", msg)
        }
    }
}

impl std::error::Error for DBError {}