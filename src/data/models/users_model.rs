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

#[derive(Deserialize, Serialize, Clone)]
pub struct NewUser {
    pub username: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub title: Option<String>,
}

#[derive(IntoValues, Clone)]
pub struct InsertableNewUser {
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
}

impl From<NewUser> for InsertableNewUser {
    fn from(new_user: NewUser) -> InsertableNewUser {
        InsertableNewUser {
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: new_user.title.or(Some(new_user.username)),
        }
    }
}

impl From<InsertableNewUser> for DbUser {
    fn from(new_user: InsertableNewUser) -> DbUser {
        DbUser {
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: new_user.title.unwrap_or(new_user.username),
        }
    }
}
