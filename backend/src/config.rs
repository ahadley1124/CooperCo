use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Settings {
    pub surrealdb_url: String,
    pub surrealdb_namespace: String,
    pub surrealdb_database: String,
    pub surrealdb_username: String,
    pub surrealdb_password: String,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            surrealdb_url: env::var("SURREALDB_URL")
                .unwrap_or_else(|_| "ws://127.0.0.1:8000/rpc".to_string()),
            surrealdb_namespace: env::var("SURREALDB_NAMESPACE")
                .context("Missing SURREALDB_NAMESPACE environment variable")?,
            surrealdb_database: env::var("SURREALDB_DATABASE")
                .context("Missing SURREALDB_DATABASE environment variable")?,
            surrealdb_username: env::var("SURREALDB_USERNAME")
                .context("Missing SURREALDB_USERNAME environment variable")?,
            surrealdb_password: env::var("SURREALDB_PASSWORD")
                .context("Missing SURREALDB_PASSWORD environment variable")?,
        })
    }
}
