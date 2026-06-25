use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Customer {
    pub id: Option<String>,
    pub name: String,
    pub contact_email: Option<String>,
}
