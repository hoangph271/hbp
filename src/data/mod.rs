use crate::utils::env::{from_env, EnvKey};

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

        loop {
            match task::block_on(user_orm::UserOrm::default().init_users_table()) {
                Ok(_) => break,
                Err(e) => {
                    log::error!("{:?}", e);
                    sleep(Duration::from_secs(10))
                }
            }
        }

        loop {
            match task::block_on(lib::post_orm::init_posts_table()) {
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
