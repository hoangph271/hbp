use rocket::async_trait;
use stargate_grpc::StargateClient;

use crate::{
    data::{lib::post_orm::PostOrm, profile_orm::ProfileOrm, user_orm::UserOrm},
    utils::env::{from_env, EnvKey},
};

use self::lib::{stargate_client_from, DbError};

pub mod lib;
pub mod models;

pub mod profile_orm;
pub mod user_orm;

pub fn init_db() {
    use async_std::task;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    spawn(|| {
        log::info!("---@ init_db()");

        // FIXME: Fix these loops

        loop {
            match task::block_on(UserOrm::default().init_table()) {
                Ok(_) => break,
                Err(e) => {
                    log::error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }
        loop {
            match task::block_on(PostOrm::default().init_table()) {
                Ok(_) => break,
                Err(e) => {
                    log::error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }
        loop {
            match task::block_on(ProfileOrm::default().init_table()) {
                Ok(_) => break,
                Err(e) => {
                    log::error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }

        log::info!("---# init_db()");
    });
}

pub struct OrmConfig {
    pub keyspace: String,
    pub astra_uri: String,
    pub bearer_token: String,
}

impl Default for OrmConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl OrmConfig {
    #[cfg(test)]
    pub fn from_test_env() -> Self {
        Self {
            keyspace: dotenv!("TEST_ASTRA_KEY_SPACE").to_owned(),
            astra_uri: dotenv!("TEST_ASTRA_URI").to_owned(),
            bearer_token: dotenv!("TEST_ASTRA_BEARER_TOKEN").to_owned(),
        }
    }

    pub fn from_env() -> Self {
        Self {
            keyspace: from_env(EnvKey::AstraKeySpace).to_owned(),
            astra_uri: from_env(EnvKey::AstraUri).to_owned(),
            bearer_token: from_env(EnvKey::AstraBearerToken).to_owned(),
        }
    }
}

#[async_trait]
pub trait OrmInit {
    fn orm_config(&self) -> &OrmConfig;
    async fn stargate_client(&self) -> Result<StargateClient, DbError> {
        let orm_config = self.orm_config();
        stargate_client_from(orm_config).await
    }

    async fn init_table(&self) -> Result<(), DbError>;

    #[cfg(test)]
    async fn drop_table(&self) -> Result<(), DbError>;

    #[cfg(test)]
    async fn reset_table(&self) -> Result<(), DbError> {
        self.drop_table().await?;
        self.init_table().await?;

        Ok(())
    }
}
