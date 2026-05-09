use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Person {
    pub id: Option<i64>,
    pub name: String,
    pub surname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonWithRoles {
    #[serde(flatten)]
    pub person: Person,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Option<i64>,
    pub name: String,
    pub delegation_hours: f64,
}

impl Person {
    pub fn new(name: String, surname: String) -> Self {
        Self {
            id: None,
            name,
            surname,
        }
    }
}

impl Role {
    pub fn new(name: String, delegation_hours: f64) -> Self {
        Self {
            id: None,
            name,
            delegation_hours,
        }
    }
}
