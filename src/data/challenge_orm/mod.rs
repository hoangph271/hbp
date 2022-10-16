// #[cfg(test)]
// mod challenge_orm_test;

use super::{OrmConfig, OrmInit};
use crate::data::lib::*;
use crate::data::models::challenges_model::Challenge as DbChallenge;
use chrono::{TimeZone, Utc};
use hbp_types::Challenge;
use httpstatus::StatusCode;
use log::{error, info};
use rocket::async_trait;
use stargate_grpc::Query;

#[derive(Default)]
pub struct ChallengeOrm {
    orm_config: OrmConfig,
}

#[async_trait]
impl OrmInit for ChallengeOrm {
    fn orm_config(&self) -> &OrmConfig {
        &self.orm_config
    }

    fn table_name(&self) -> String {
        "challenges".to_owned()
    }

    async fn init_table(&self) -> Result<(), DbError> {
        let create_challenges_table = stargate_grpc::Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "CREATE TABLE IF NOT EXISTS challenges (
                    id text PRIMARY KEy,
                    title text,
                    why text,
                    note text,
                    started_at timestamp,
                    end_at timestamp,
                    finished boolean,
                )",
            )
            .build();

        self.stargate_client()
            .await?
            .execute_query(create_challenges_table)
            .await
            .map_err(|e| {
                let msg = format!("init_table() challenges failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        info!("created create_challenges_table table");
        Ok(())
    }
}

impl ChallengeOrm {
    pub async fn find(&self) -> Result<Vec<Challenge>, DbError> {
        let challenges_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM challenges")
            .build();

        let maybe_challenges: Option<Vec<DbChallenge>> =
            execute_stargate_query_for_vec(challenges_query).await?;

        Ok(maybe_challenges
            .unwrap_or_default()
            .into_iter()
            .map(map_challenge)
            .collect())
    }

    pub async fn find_one(&self, id: &str) -> Result<Option<Challenge>, DbError> {
        let challenge_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM challenges WHERE id = :id")
            .bind_name("id", id)
            .build();

        let client = self.stargate_client().await?;
        let maybe_challenge: Option<Challenge> =
            execute_stargate_query_for_one(client, challenge_query)
                .await?
                .map(map_challenge);

        Ok(maybe_challenge)
    }

    pub async fn create(&self, new_challenge: Challenge) -> Result<Challenge, DbError> {
        let insert_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                INSERT INTO challenges(id, title, why, note, started_at, end_at, finished)
                VALUES (:id, :title, :why, :note, :started_at, :end_at, :finished)
                IF NOT EXISTS",
            )
            .bind_name("id", new_challenge.id.clone())
            .bind_name("title", new_challenge.title)
            .bind_name("why", new_challenge.why)
            .bind_name("note", new_challenge.note)
            .bind_name("started_at", new_challenge.start_at_ms.timestamp_millis())
            .bind_name("end_at", new_challenge.end_at_ms.timestamp_millis())
            .bind_name("finished", new_challenge.finished)
            .build();

        let client = self.stargate_client().await?;
        let mut result_set = execute_stargate_query(client, insert_query)
            .await?
            .unwrap_or_else(|| panic!("result_set must NOT be None"));

        let mut row = result_set
            .rows
            .pop()
            .unwrap_or_else(|| panic!("result_set MUST has one row"));
        let inserted: bool = row.try_take(0).map_err(|e| {
            let message = format!("Can't read inserted: {e}");

            error!("{message}");

            DbError::internal_server_error(message)
        })?;

        if inserted {
            match self.find_one(&new_challenge.id).await? {
                Some(challenge) => Ok(challenge),
                None => Err(DbError::internal_server_error(
                    "create_challenge failed".to_owned(),
                )),
            }
        } else {
            Err(DbError {
                status_code: StatusCode::Conflict,
                message: format!("challenge :id `{}` existed", new_challenge.id),
            })
        }
    }

    pub async fn update(&self, challenge: Challenge) -> Result<Challenge, DbError> {
        let update_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                UPDATE challenges
                SET title = :title, why = :why, note = :note, started_at = :started_at, end_at = :end_at, finished = :finished
                WHERE id = :id
                IF EXISTS",
            )
            .bind_name("id", challenge.id.clone())
            .bind_name("title", challenge.title)
            .bind_name("why", challenge.why)
            .bind_name("note", challenge.note)
            .bind_name("started_at", challenge.start_at_ms.timestamp_millis())
            .bind_name("end_at", challenge.end_at_ms.timestamp_millis())
            .bind_name("finished", challenge.finished)
            .build();

        let client = self.stargate_client().await?;
        let mut rs = execute_stargate_query(client, update_query)
            .await?
            .ok_or_else(|| DbError {
                status_code: StatusCode::NotFound,
                message: format!("Challenge NotFound: {}", challenge.id),
            })?;

        let is_found: bool = rs.rows.pop().unwrap().try_take(0).unwrap();

        if !is_found {
            Err(DbError {
                status_code: StatusCode::NotFound,
                message: format!("Challenge NotFound: {}", challenge.id),
            })?
        }

        match self.find_one(&challenge.id).await? {
            Some(challenge) => Ok(challenge),
            None => Err(DbError::internal_server_error(
                "create_challenge failed".to_owned(),
            )),
        }
    }

    pub async fn delete(&self, id: &str) -> Result<(), DbError> {
        let delete_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                DELETE FROM challenges
                WHERE id = :id",
            )
            .bind_name("id", id)
            .build();

        let client = self.stargate_client().await?;
        execute_stargate_query(client, delete_query).await?;

        Ok(())
    }
}

fn map_challenge(db_challenge: DbChallenge) -> Challenge {
    Challenge {
        id: db_challenge.id,
        title: db_challenge.title,
        why: db_challenge.why,
        note: db_challenge.note,
        start_at_ms: Utc.timestamp_millis(db_challenge.started_at),
        end_at_ms: Utc.timestamp_millis(db_challenge.end_at),
        finished: db_challenge.finished,
    }
}
