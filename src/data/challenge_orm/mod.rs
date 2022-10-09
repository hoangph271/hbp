// #[cfg(test)]
// mod challenge_orm_test;

use crate::data::{lib::*, models::users_model::*};
use httpstatus::StatusCode;
use log::{error, info};
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
    pub async fn find_one(&self, id: &str) -> Result<Option<DbUser>, DbError> {
        let challenge_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM challenges WHERE id = :id")
            .bind_name("id", id)
            .build();

        let client = self.stargate_client().await?;
        let maybe_user: Option<DbUser> =
            execute_stargate_query_for_one(client, challenge_query).await?;

        Ok(maybe_user)
    }

    // pub async fn create_one(&self, new_user: DbUser) -> Result<DbUser, DbError> {
    //     let insert_query = Query::builder()
    //         .keyspace(&self.orm_config.keyspace)
    //         .query(
    //             "
    //             INSERT INTO users(username, hashed_password, title)
    //             VALUES (:username, :hashed_password, :title)
    //             IF NOT EXISTS",
    //         )
    //         .bind(new_user.clone())
    //         .build();

    //     let client = self.stargate_client().await?;
    //     let mut result_set = execute_stargate_query(client, insert_query)
    //         .await?
    //         .unwrap_or_else(|| panic!("result_set must NOT be None"));

    //     let mut row = result_set
    //         .rows
    //         .pop()
    //         .unwrap_or_else(|| panic!("result_set MUST has one row"));
    //     let inserted: bool = row.try_take(0).map_err(|e| {
    //         let message = format!("Can't read inserted: {e}");

    //         error!("{message}");

    //         DbError::internal_server_error(message)
    //     })?;

    //     if inserted {
    //         match self.find_one(&new_user.username).await? {
    //             Some(user) => Ok(user),
    //             None => Err(DbError::internal_server_error(
    //                 "create_user failed".to_owned(),
    //             )),
    //         }
    //     } else {
    //         Err(DbError {
    //             status_code: StatusCode::Conflict,
    //             message: format!("username `{}` existed", new_user.username),
    //         })
    //     }
    // }

    // pub async fn update_user(&self, user: PutUser) -> Result<(), DbError> {
    //     let user_query = Query::builder()
    //         .keyspace(&self.orm_config.keyspace)
    //         .query(
    //             "
    //             UPDATE users
    //             SET title = :title
    //             WHERE username = :username",
    //         )
    //         .bind(user.clone())
    //         .build();

    //     let client = self.stargate_client().await?;
    //     let _ = execute_stargate_query(client, user_query).await?;

    //     Ok(())
    // }
}
