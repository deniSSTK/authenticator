use std::env;

pub(crate) struct Config {
    pub(crate) database_url: String,
    pub(crate) key_encryption_key: String,
    pub(crate) bind_addr: String,
}

impl Config {
    pub(crate) fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            key_encryption_key: env::var("KEY_ENCRYPTION_KEY")?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into()),
        })
    }
}
