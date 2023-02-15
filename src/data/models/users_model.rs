use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct DbUser {
    pub username: String,
    pub hashed_password: String,
    pub title: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct User {
    pub username: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct PutUser {
    pub username: String,
    pub title: String,
}
