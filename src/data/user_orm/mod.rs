#[cfg(test)]
mod user_orm_test;

use crate::{
    data::{lib::*, models::users_model::*},
    utils::env::{from_env, EnvKey},
};
use httpstatus::StatusCode;
use stargate_grpc::{Query, StargateClient};

pub struct UserOrm {
    pub keyspace: String,
    pub astra_uri: String,
    pub bearer_token: String,
}

impl UserOrm {
    pub fn from_env() -> Self {
        Self {
            keyspace: from_env(EnvKey::AstraKeySpace).to_owned(),
            astra_uri: from_env(EnvKey::AstraUri).to_owned(),
            bearer_token: from_env(EnvKey::AstraBearerToken).to_owned(),
        }
    }

    async fn build_stargate_client(&self) -> Result<StargateClient, DbError> {
        build_stargate_client(&self.astra_uri, &self.bearer_token).await
    }

    pub async fn init_users_table(&self) -> Result<(), DbError> {
        let create_users_table = stargate_grpc::Query::builder()
            .keyspace(&self.keyspace)
            .query(
                "CREATE TABLE IF NOT EXISTS users (
                    username text PRIMARY KEY,
                    hashed_password text,
                    title text,
                )",
            )
            .build();

        self.build_stargate_client()
            .await?
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("init_users_table failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("created users table");
        Ok(())
    }
    #[cfg(test)]
    #[allow(dead_code)]
    pub async fn drop_users_table(&self) -> Result<(), DbError> {
        let create_users_table = stargate_grpc::Query::builder()
            .keyspace(&self.keyspace)
            .query("DROP TABLE IF EXISTS users")
            .build();

        self.build_stargate_client()
            .await?
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("drop_users_table failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("created users table");
        Ok(())
    }

    pub async fn find_one(&self, username: &str) -> Result<Option<User>, DbError> {
        let user_query = Query::builder()
            .keyspace(&self.keyspace)
            .query("SELECT * FROM users WHERE username = :username")
            .bind_name("username", username)
            .build();

        let client = self.build_stargate_client().await?;
        let maybe_user: Option<User> = execute_stargate_query_for_one(client, user_query).await?;

        Ok(maybe_user)
    }

    pub async fn create_user(&self, new_user: NewUser) -> Result<User, DbError> {
        let new_user: InsertableNewUser = new_user.into();

        let user_query = Query::builder()
            .keyspace(&self.keyspace)
            .query(
                "INSERT INTO users(username, hashed_password, title) \
                        VALUES (:username, :hashed_password, :title) \
                        IF NOT EXISTS",
            )
            .bind(new_user.clone())
            .build();

        let client = self.build_stargate_client().await?;
        let mut result_set = execute_stargate_query(client, user_query).await?.unwrap();

        let mut row = result_set.rows.pop().unwrap();
        let inserted: bool = row.try_take(0).unwrap();

        if inserted {
            Ok(new_user.into())
        } else {
            Err(DbError {
                status_code: StatusCode::Conflict,
                message: format!("username `{}` existed", new_user.username),
            })
        }
    }
}
