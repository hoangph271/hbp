use serde::Serialize;
use stargate_grpc_derive::{IntoValues, TryFromRow};

use super::users_model::DbUser;

#[derive(Serialize, TryFromRow, Clone, IntoValues)]
pub struct DbProfile {
    pub username: String,
    pub title: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub description: Option<String>,
}

impl DbProfile {
    pub fn from_username(username: String) -> DbProfile {
        DbProfile {
            username: username.clone(),
            title: username,
            avatar_url: None,
            description: None,
        }
    }
}

impl From<DbUser> for DbProfile {
    fn from(db_user: DbUser) -> Self {
        DbProfile {
            username: db_user.username,
            title: db_user.title,
            avatar_url: None,
            description: None,
        }
    }
}
