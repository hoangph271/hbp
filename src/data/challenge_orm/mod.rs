// #[cfg(test)]
// mod challenge_orm_test;

use crate::data::{lib::*, models::challenges_model::*};
use chrono::{DateTime, NaiveDateTime, Utc};
use log::info;
use rocket::async_trait;
use stargate_grpc::Query;

use super::{OrmConfig, OrmInit};

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
    pub async fn find(&self) -> Result<Vec<hbp_types::Challenge>, DbError> {
        let challenges_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM challenges")
            .build();

        let maybe_challenges: Option<Vec<Challenge>> =
            execute_stargate_query_for_vec(challenges_query).await?;

        Ok(maybe_challenges
            .unwrap_or_default()
            .into_iter()
            .map(map_challenge)
            .collect())
    }

    pub async fn find_one(&self, id: &str) -> Result<Option<hbp_types::Challenge>, DbError> {
        let challenge_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM challenges WHERE id = :id")
            .bind_name("id", id)
            .build();

        let client = self.stargate_client().await?;
        let maybe_challenge: Option<hbp_types::Challenge> =
            execute_stargate_query_for_one(client, challenge_query)
                .await?
                .map(map_challenge);

        Ok(maybe_challenge)
    }
}

fn map_challenge(challenge: Challenge) -> hbp_types::Challenge {
    hbp_types::Challenge {
        id: challenge.id,
        title: challenge.title,
        why: challenge.why,
        note: challenge.note,
        started_at: DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(challenge.started_at, 0),
            Utc,
        ),
        end_at: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(challenge.end_at, 0), Utc),
        finished: challenge.finished,
    }
}
