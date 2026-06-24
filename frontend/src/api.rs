use gloo_net::http::Request;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Customer {
    pub id: Option<String>,
    pub name: String,
    pub contact_email: Option<String>,
}

pub async fn fetch_health() -> Result<String, String> {
    let response = Request::get("/api/health")
        .send()
        .await
        .map_err(|err| err.to_string())?;

    response.text().await.map_err(|err| err.to_string())
}

pub async fn fetch_customers() -> Result<Vec<Customer>, String> {
    let response = Request::get("/api/customers")
        .send()
        .await
        .map_err(|err| err.to_string())?;

    response
        .json::<Vec<Customer>>()
        .await
        .map_err(|err| err.to_string())
}
