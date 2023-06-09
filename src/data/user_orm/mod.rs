#[cfg(test)]
mod user_orm_test;

use crate::data::{lib::*, models::users_model::*};
use rocket::async_trait;
use serde::__private::from_utf8_lossy;

use super::OrmInit;

#[derive(Default)]
pub struct UserOrm {}

#[async_trait]
impl OrmInit for UserOrm {
    #[cfg(not(test))]
    fn db_file_name(&self) -> String {
        "users.sled.db".to_string()
    }
}

impl UserOrm {
    pub async fn find_one(&self, db: &sled::Db, username: &str) -> Result<Option<DbUser>, DbError> {
        if let Some(raw) = db.get(username).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            let user: DbUser = serde_json::from_str(&json).unwrap();

            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub async fn create_user(&self, db: &sled::Db, new_user: DbUser) -> Result<DbUser, DbError> {
        let username = new_user.username.clone();

        db.insert(
            new_user.username.clone(),
            serde_json::to_string(&new_user).unwrap().as_bytes(),
        )
        .unwrap();

        Ok(self.find_one(db, &username).await.unwrap().unwrap())
    }

    pub async fn update_user(&self, _user: PutUser) -> Result<(), DbError> {
        // let user_query = Query::builder()
        //     .keyspace(&self.orm_config.keyspace)
        //     .query(
        //         "
        //         UPDATE users
        //         SET title = :title
        //         WHERE username = :username",
        //     )
        //     .bind(user.clone())
        //     .build();

        // let client = self.stargate_client().await?;
        // let _ = execute_stargate_query(client, user_query).await?;

        // Ok(())
        todo!()
    }
}
