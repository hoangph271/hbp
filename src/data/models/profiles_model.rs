use serde::{Deserialize, Serialize};

use super::users_model::DbUser;

#[derive(Serialize, Clone, Deserialize)]
pub struct DbProfile {
    pub username: String,
    pub title: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub description: Option<String>,
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
