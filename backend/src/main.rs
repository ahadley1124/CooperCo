mod config;
mod db;
mod models;
mod routes;

use anyhow::{Context, Result};
use config::Settings;
use db::DbClient;
use rocket::{build, routes};

#[rocket::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::from_env().context("Failed to read backend settings")?;
    let db = DbClient::connect(&settings)
        .await
        .context("Failed to connect to SurrealDB")?;

    build()
        .manage(db)
        .mount("/api", routes![routes::health, routes::customers])
        .launch()
        .await
        .context("Rocket server failed to start")?;

    Ok(())
}
