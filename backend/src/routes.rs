use crate::db::DbClient;
use crate::models::Customer;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

#[get("/health")]
pub async fn health() -> Json<&'static str> {
    Json("ok")
}

#[get("/customers")]
pub async fn customers(db: &State<DbClient>) -> Result<Json<Vec<Customer>>, Status> {
    db.list_customers()
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}
