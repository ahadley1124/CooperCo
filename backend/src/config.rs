use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub surrealdb_url: String,
    pub surrealdb_namespace: String,
    pub surrealdb_database: String,
    pub surrealdb_path: String,
    pub surrealdb_username: Option<String>,
    pub surrealdb_password: Option<String>,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            surrealdb_url: env::var("SURREALDB_URL")
                .unwrap_or_else(|_| "ws://127.0.0.1:8000/rpc".to_string()),
            surrealdb_namespace: env::var("SURREALDB_NAMESPACE")
                .unwrap_or_else(|_| "cooperco".to_string()),
            surrealdb_database: env::var("SURREALDB_DATABASE")
                .unwrap_or_else(|_| "app".to_string()),
            surrealdb_path: env::var("SURREALDB_PATH")
                .unwrap_or_else(|_| ".surrealdb".to_string()),
            surrealdb_username: env::var("SURREALDB_USERNAME").ok(),
            surrealdb_password: env::var("SURREALDB_PASSWORD").ok(),
        })
    }

    pub fn uses_remote_surrealdb(&self) -> bool {
        !self.surrealdb_url.is_empty()
            && self.surrealdb_username.is_some()
            && self.surrealdb_password.is_some()
    }
}
