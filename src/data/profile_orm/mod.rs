#[cfg(test)]
mod profile_orm_test;

use rocket::async_trait;

use super::{lib::DbError, OrmConfig, OrmInit};

#[derive(Default)]
pub struct ProfileOrm {
    pub orm_config: OrmConfig,
}

impl ProfileOrm {
    #[allow(dead_code)]
    pub async fn find_one(&self, _username: &str) -> Result<(), DbError> {
        let _client = self.stargate_client().await?;

        todo!()
    }
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
                    avatarUrl text,
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
