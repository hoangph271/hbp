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
    pub async fn find_one(&self, username: &str) -> Result<Option<DbProfile>, DbError> {
        let sled = sled::open(self.db_file_name()).unwrap();

        if let Some(raw) = sled.get(username).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            Ok(serde_json::from_str(&json).ok())
        } else {
            Ok(None)
        }
    }

    pub async fn create_profile(&self, new_profile: DbProfile) -> Result<DbProfile, DbError> {
        let sled = sled::open(self.db_file_name()).unwrap();
        let username = new_profile.username.clone();

        sled.insert(
            new_profile.username.clone(),
            serde_json::to_string(&new_profile).unwrap().as_bytes(),
        ).unwrap();

        self.find_one(&username)
            .await
            .unwrap()
            .ok_or(DbError::internal_server_error(
                "create Challenge failed".to_string(),
            ))
    }
}
