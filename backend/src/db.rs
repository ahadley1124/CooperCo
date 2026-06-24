use crate::config::Settings;
use crate::models::Customer;
use anyhow::Result;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};

#[derive(Clone)]
pub struct DbClient {
    db: Surreal<Client>,
}

impl DbClient {
    pub async fn connect(settings: &Settings) -> Result<Self> {
        let db: Surreal<Client> = Surreal::<Client>::new::<Ws>(&settings.surrealdb_url).await?;

        db.signin(Root {
            username: &settings.surrealdb_username,
            password: &settings.surrealdb_password,
        })
        .await?;

        db.use_ns(&settings.surrealdb_namespace)
            .use_db(&settings.surrealdb_database)
            .await?;

        Ok(Self { db })
    }

    pub async fn list_customers(&self) -> Result<Vec<Customer>> {
        let customers: Vec<Customer> = self
            .db
            .query("SELECT * FROM customer LIMIT 10")
            .await?
            .take::<Vec<Customer>>(0)?;
        Ok(customers)
    }
}
