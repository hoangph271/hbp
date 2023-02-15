#[cfg(test)]
mod profile_orm_test;

use rocket::async_trait;
use serde::__private::from_utf8_lossy;

use super::{lib::DbError, models::profiles_model::DbProfile, OrmInit};

#[derive(Default)]
pub struct ProfileOrm {}

#[async_trait]
impl OrmInit for ProfileOrm {
    fn db_file_name(&self) -> String {
        "profiles.sled.db".to_owned()
    }
}

impl ProfileOrm {
    #[allow(dead_code)]
    pub async fn find_one(&self, username: &str, db: &sled::Db) -> Result<Option<DbProfile>, DbError> {
        if let Some(raw) = db.get(username).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            Ok(serde_json::from_str(&json).ok())
        } else {
            Ok(None)
        }
    }

    pub async fn create_profile(&self, new_profile: DbProfile, db: &sled::Db) -> Result<DbProfile, DbError> {
        let username = new_profile.username.clone();

        db.insert(
            new_profile.username.clone(),
            serde_json::to_string(&new_profile).unwrap().as_bytes(),
        ).unwrap();

        self.find_one(&username, db)
            .await
            .unwrap()
            .ok_or(DbError::internal_server_error(
                "create Challenge failed".to_string(),
            ))
    }
}
