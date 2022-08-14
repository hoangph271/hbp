#[cfg(test)]
mod user_orm_test;

use crate::data::{lib::*, models::users_model::*};
use httpstatus::StatusCode;
use stargate_grpc::{Query, StargateClient};

pub struct UserOrm {
    pub keyspace: String,
}

impl UserOrm {
    pub fn from_env() -> Self {
        Self {
            keyspace: get_keyspace().to_owned(),
        }
    }

    fn build_stargate_client(&self) -> Result<StargateClient, DbError> {
        todo!()
    }

    pub async fn init_users_table(&self) -> Result<(), DbError> {
        let mut client = self.build_stargate_client()?;

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

        client
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("init_users_table failed at .execute_query(): {e:?}");

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

        let client = self.build_stargate_client()?;
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

        let mut result_set = execute_stargate_query(user_query).await?.unwrap();
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
