#[cfg(test)]
mod profile_orm_test;

use httpstatus::StatusCode;
use rocket::async_trait;
use stargate_grpc::Query;

use super::{
    lib::{execute_stargate_query, execute_stargate_query_for_one, DbError},
    models::profiles_model::DbProfile,
    OrmConfig, OrmInit,
};

#[derive(Default)]
pub struct ProfileOrm {
    pub orm_config: OrmConfig,
}

#[async_trait]
impl OrmInit for ProfileOrm {
    fn orm_config(&self) -> &OrmConfig {
        &self.orm_config
    }

    async fn init_table(&self) -> Result<(), DbError> {
        let create_profiles_table = stargate_grpc::Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "CREATE TABLE IF NOT EXISTS profiles (
                    username text PRIMARY KEY,
                    title text,
                    avatar_url text,
                )",
            )
            .build();

        self.stargate_client()
            .await?
            .execute_query(create_profiles_table)
            .await
            .map_err(|e| {
                let msg = format!("init_table() profiles failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("created profiles table");
        Ok(())
    }

    #[cfg(test)]
    async fn drop_table(&self) -> Result<(), DbError> {
        let create_users_table = stargate_grpc::Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("DROP TABLE IF EXISTS profiles")
            .build();

        self.stargate_client()
            .await?
            .execute_query(create_users_table)
            .await
            .map_err(|e| {
                let msg = format!("drop_table() profiles failed at .execute_query(): {e:?}");

                DbError::internal_server_error(msg)
            })?;

        println!("dropped profiles table");
        Ok(())
    }
}

impl ProfileOrm {
    #[allow(dead_code)]
    pub async fn find_one(&self, username: &str) -> Result<Option<DbProfile>, DbError> {
        let profile_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query("SELECT * FROM profiles WHERE username = :username")
            .bind_name("username", username)
            .build();

        let client = self.stargate_client().await?;
        let maybe_profile: Option<DbProfile> =
            execute_stargate_query_for_one(client, profile_query).await?;

        Ok(maybe_profile)
    }

    pub async fn create_profile(&self, new_profile: DbProfile) -> Result<DbProfile, DbError> {
        let insert_query = Query::builder()
            .keyspace(&self.orm_config.keyspace)
            .query(
                "
                INSERT INTO profiles(username, title, avatar_url)
                VALUES (:username, :title, :avatar_url)
                IF NOT EXISTS",
            )
            .bind(new_profile.clone())
            .build();

        let client = self.stargate_client().await?;
        let mut result_set = execute_stargate_query(client, insert_query).await?.unwrap();

        let mut row = result_set.rows.pop().unwrap();
        let inserted: bool = row.try_take(0).unwrap();

        if inserted {
            match self.find_one(&new_profile.username).await? {
                Some(user) => Ok(user),
                None => Err(DbError::internal_server_error(
                    "create_user failed".to_owned(),
                )),
            }
        } else {
            Err(DbError {
                status_code: StatusCode::Conflict,
                message: format!("username `{}` existed", new_profile.username),
            })
        }
    }
}
