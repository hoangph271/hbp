// #[cfg(test)]
// mod challenge_orm_test;

use super::OrmInit;
use crate::data::lib::*;
use hbp_types::Challenge;
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

    pub async fn find_one(&self, id: &str, db: &sled::Db) -> Result<Option<Challenge>, DbError> {
        if let Some(raw) = db.get(id).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            Ok(serde_json::from_str(&json).ok())
        } else {
            Ok(None)
        }
    }

    pub async fn create(&self, new_challenge: Challenge, db: &sled::Db) -> Result<Challenge, DbError> {
        let id = new_challenge.id.clone();

        db.insert(
            new_challenge.id.clone(),
            serde_json::to_string(&new_challenge).unwrap().as_bytes(),
        ).unwrap();

        self.find_one(&id, &db)
            .await
            .unwrap()
            .ok_or(DbError::internal_server_error(
                "create Challenge failed".to_string(),
            ))
    }

    pub async fn update(&self, _challenge: Challenge, db: &sled::Db) -> Result<Challenge, DbError> {
        todo!()
        // let update_query = Query::builder()
        //     .keyspace(&self.orm_config.keyspace)
        //     .query(
        //         "
        //         UPDATE challenges
        //         SET title = :title, why = :why, note = :note, started_at = :started_at, end_at = :end_at, finished = :finished
        //         WHERE id = :id
        //         IF EXISTS",
        //     )
        //     .bind_name("id", challenge.id.clone())
        //     .bind_name("title", challenge.title)
        //     .bind_name("why", challenge.why)
        //     .bind_name("note", challenge.note)
        //     .bind_name("started_at", challenge.start_at_ms.timestamp_millis())
        //     .bind_name("end_at", challenge.end_at_ms.timestamp_millis())
        //     .bind_name("finished", challenge.finished)
        //     .build();

        // let client = self.stargate_client().await?;
        // let mut rs = execute_stargate_query(client, update_query)
        //     .await?
        //     .ok_or_else(|| DbError {
        //         status_code: StatusCode::NotFound,
        //         message: format!("Challenge NotFound: {}", challenge.id),
        //     })?;

        // let is_found: bool = rs.rows.pop().unwrap().try_take(0).unwrap();

        // if !is_found {
        //     Err(DbError {
        //         status_code: StatusCode::NotFound,
        //         message: format!("Challenge NotFound: {}", challenge.id),
        //     })?
        // }

        // match self.find_one(&challenge.id).await? {
        //     Some(challenge) => Ok(challenge),
        //     None => Err(DbError::internal_server_error(
        //         "create_challenge failed".to_owned(),
        //     )),
        // }
    }

    pub async fn delete(&self, id: &str, db: &sled::Db) -> Result<(), DbError> {
        db.remove(id).unwrap();

        Ok(())
    }
}
