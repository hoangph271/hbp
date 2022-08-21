use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stargate_grpc_derive::{IntoValues, TryFromRow};

#[derive(Debug, Serialize, TryFromRow, Deserialize, JsonSchema, Clone, IntoValues)]
pub struct DbUser {
    pub username: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub title: String,
}

#[derive(Debug, Serialize, TryFromRow, Deserialize, JsonSchema, Clone, IntoValues)]
pub struct PutUser {
    pub username: String,
    pub title: String,
}
