#[cfg(test)]
mod user_orm_test;

use crate::data::{lib::*, models::users_model::*};
use httpstatus::StatusCode;
use rocket::async_trait;
use stargate_grpc::Query;

use super::{OrmConfig, OrmInit};

#[derive(Default)]
pub struct UserOrm {
    orm_config: OrmConfig,
}

#[async_trait]
impl OrmInit for UserOrm {
    fn orm_config(&self) -> &OrmConfig {
        &self.orm_config
    }

    async fn init_table(&self) -> Result<(), DbError> {
        let create_users_table = stargate_grpc::Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "CREATE TABLE IF NOT EXISTS users (
                    username text PRIMARY KEY,
                    hashed_password text,
                    title text,
                )",
            )
            .build();

        self.stargate_client()
            .await?
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("init_table() users failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("created users table");
        Ok(())
    }

    #[cfg(test)]
    async fn drop_table(&self) -> Result<(), DbError> {
        let create_users_table = stargate_grpc::Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("DROP TABLE IF EXISTS users")
            .build();

        self.stargate_client()
            .await?
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("drop_table() users failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("dropped users table");
        Ok(())
    }
}

impl UserOrm {
    pub async fn find_one(&self, username: &str) -> Result<Option<DbUser>, DbError> {
        let user_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM users WHERE username = :username")
            .bind_name("username", username)
            .build();

        let client = self.stargate_client().await?;
        let maybe_user: Option<DbUser> = execute_stargate_query_for_one(client, user_query).await?;

        Ok(maybe_user)
    }

    pub async fn create_user(&self, new_user: NewUser) -> Result<DbUser, DbError> {
        let new_user: InsertableNewUser = new_user.into();

        let user_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                INSERT INTO users(username, hashed_password, title)
                VALUES (:username, :hashed_password, :title)
                IF NOT EXISTS",
            )
            .bind(new_user.clone())
            .build();

        let client = self.stargate_client().await?;
        let mut result_set = execute_stargate_query(client, user_query).await?.unwrap();

        let mut row = result_set.rows.pop().unwrap();
        let inserted: bool = row.try_take(0).unwrap();

        if inserted {
            match self.find_one(&new_user.username).await? {
                Some(user) => Ok(user),
                None => Err(DbError::internal_server_error(
                    "create_user failed".to_owned(),
                )),
            }
        } else {
            Err(DbError {
                status_code: StatusCode::Conflict,
                message: format!("username `{}` existed", new_user.username),
            })
        }
    }

    pub async fn update_user(&self, user: PutUser) -> Result<(), DbError> {
        let user_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                UPDATE users
                SET title = :title
                WHERE username = :username",
            )
            .bind(user.clone())
            .build();

        let client = self.stargate_client().await?;
        let res = execute_stargate_query(client, user_query).await?;
        println!("{res:?}");

        Ok(())
    }
}
