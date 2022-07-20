use serde::{Deserialize, Serialize};
use stargate_grpc_derive::{IntoValues, TryFromRow};

#[derive(Debug, Serialize, TryFromRow, Deserialize)]
pub struct User {
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
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
            title: Some(new_user.username),
        }
    }
}

impl From<InsertableNewUser> for User {
    fn from(new_user: InsertableNewUser) -> User {
        User {
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: Some(new_user.username),
        }
    }
}
