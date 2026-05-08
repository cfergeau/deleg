use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Person {
    pub id: Option<i64>,
    pub name: String,
    pub surname: String,
    pub role: String,
}

impl Person {
    pub fn new(name: String, surname: String, role: String) -> Self {
        Self {
            id: None,
            name,
            surname,
            role,
        }
    }
}
