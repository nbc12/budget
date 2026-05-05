use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Card {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateCardRequest {
    pub name: String,
    pub is_active: bool,
}
