use serde::{Deserialize, Serialize};
use stargate_grpc_derive::{IntoValues, TryFromRow};

#[derive(Debug, Serialize, TryFromRow, Deserialize)]
pub struct User {
    pub username: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub title: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NewUser {
    pub username: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub title: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(IntoValues, Clone)]
pub struct InsertableNewUser {
    pub username: String,
    pub hashed_password: String,
    pub title: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<NewUser> for InsertableNewUser {
    fn from(new_user: NewUser) -> InsertableNewUser {
        InsertableNewUser {
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: new_user.title.or(Some(new_user.username)),
            avatar_url: new_user.avatar_url,
        }
    }
}

impl From<InsertableNewUser> for User {
    fn from(new_user: InsertableNewUser) -> User {
        User {
            username: new_user.username.to_owned(),
            hashed_password: new_user.hashed_password.to_owned(),
            title: new_user.title.unwrap_or(new_user.username),
            avatar_url: None,
        }
    }
}
