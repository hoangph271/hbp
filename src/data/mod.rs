use crate::utils::env::is_prod;
use async_std::{task, fs};
use rocket::async_trait;
use std::future::Future;
use std::thread::sleep;
use std::time::Duration;

use self::lib::DbError;

pub mod lib;
pub mod models;

pub mod challenge_orm;
pub mod profile_orm;
pub mod user_orm;

fn block_init_table<F, Output>(executor: F)
where
    F: Fn() -> Output,
    Output: Future<Output = Result<(), DbError>>,
{
    const RETRY_LIMIT: usize = 5;
    let mut count = 0;

    loop {
        if count == RETRY_LIMIT {
            break;
        }

        match task::block_on(executor()) {
            Ok(_) => break,
            Err(e) => {
                count += 1;
                log::error!("{:?}", e);
                sleep(Duration::from_secs(10))
            }
        }
    }
}

pub fn init_db() {
    if is_prod() {
        return;
    }

    use std::thread::spawn;

    spawn(|| {
        log::info!("---@ init_db()");

        // block_init_table(|| async { user_orm::UserOrm::default().init_table().await });
        // block_init_table(|| async { post_orm::PostOrm::default().init_table().await });
        // block_init_table(|| async { profile_orm::ProfileOrm::default().init_table().await });
        block_init_table(|| async { challenge_orm::ChallengeOrm::default().init_table().await });

        log::info!("---# init_db()");
    });
}

pub struct OrmConfig {}

impl Default for OrmConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl OrmConfig {
    // #[cfg(test)]
    // pub fn from_test_env() -> Self {
    //     Self {}
    // }

    pub fn from_env() -> Self {
        Self {}
    }
}

#[async_trait]
pub trait OrmInit {
    fn db_file_name(&self) -> String;

    async fn init_table(&self) -> Result<(), DbError> {
        fs::File::create(self.db_file_name()).await.unwrap();

        Ok(())
    }

    // #[cfg(test)]
    // async fn drop_table(&self) -> Result<(), DbError>;

    // #[cfg(test)]
    // async fn reset_table(&self) -> Result<(), DbError> {
    //     self.drop_table().await?;
    //     self.init_table().await?;

    //     Ok(())
    // }
}
