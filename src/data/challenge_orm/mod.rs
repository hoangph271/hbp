use super::{OrmInit};
use crate::{data::lib::*, shared::Challenge};
use rocket::async_trait;
use serde::__private::from_utf8_lossy;

#[derive(Default)]
pub struct ChallengeOrm {}

#[async_trait]
impl OrmInit for ChallengeOrm {
    fn db_file_name(&self) -> String {
        "challenges.sled.db".to_owned()
    }
}

impl ChallengeOrm {
    pub async fn find(&self, db: &sled::Db) -> Result<Vec<Challenge>, DbError> {
        let challenges: Vec<Challenge> = db
            .iter()
            .map(|raw| {
                let (_, value) = raw.unwrap();
                let json = from_utf8_lossy(&value[..]);

                serde_json::from_str(&json).unwrap()
            })
            .collect();

        Ok(challenges)
    }

    pub async fn find_one(&self, db: &sled::Db, id: &str) -> Result<Option<Challenge>, DbError> {
        if let Some(raw) = db.get(id).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            Ok(serde_json::from_str(&json).ok())
        } else {
            Ok(None)
        }
    }

    pub async fn create(
        &self,
        db: &sled::Db,
        new_challenge: Challenge,
    ) -> Result<Challenge, DbError> {
        let id = new_challenge.id.clone();

        db.insert(
            new_challenge.id.clone(),
            serde_json::to_string(&new_challenge).unwrap().as_bytes(),
        )
        .unwrap();

        self.find_one(db, &id)
            .await
            .unwrap()
            .ok_or(DbError::internal_server_error(
                "create Challenge failed".to_string(),
            ))
    }

    pub async fn update(
        &self,
        _db: &sled::Db,
        _challenge: Challenge,
    ) -> Result<Challenge, DbError> {
        todo!()
    }

    pub async fn delete(&self, db: &sled::Db, id: &str) -> Result<(), DbError> {
        db.remove(id).unwrap();

        Ok(())
    }
}
