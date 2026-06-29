use crate::config::Settings;
use crate::models::Customer;
use anyhow::Result;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

#[derive(Clone)]
pub enum DbClient {
    Remote(Surreal<Client>),
    Embedded(Surreal<Db>),
}

impl DbClient {
    pub async fn connect(settings: &Settings) -> Result<Self> {
        if settings.uses_remote_surrealdb() {
            let db: Surreal<Client> = Surreal::<Client>::new::<Ws>(&settings.surrealdb_url).await?;

            let username = settings
                .surrealdb_username
                .as_deref()
                .expect("remote database username is required");
            let password = settings
                .surrealdb_password
                .as_deref()
                .expect("remote database password is required");

            db.signin(Root { username, password }).await?;

            db.use_ns(&settings.surrealdb_namespace)
                .use_db(&settings.surrealdb_database)
                .await?;

            Ok(Self::Remote(db))
        } else {
            let db = Surreal::<Db>::new::<SurrealKv>(&settings.surrealdb_path).await?;

            db.use_ns(&settings.surrealdb_namespace)
                .use_db(&settings.surrealdb_database)
                .await?;

            Ok(Self::Embedded(db))
        }
    }

    pub async fn list_customers(&self) -> Result<Vec<Customer>> {
        match self {
            Self::Remote(db) => {
                let customers: Vec<Customer> = db
                    .query("SELECT * FROM customer LIMIT 10")
                    .await?
                    .take::<Vec<Customer>>(0)?;
                Ok(customers)
            }
            Self::Embedded(db) => {
                let customers: Vec<Customer> = db
                    .query("SELECT * FROM customer LIMIT 10")
                    .await?
                    .take::<Vec<Customer>>(0)?;
                Ok(customers)
            }
        }
    }
}
