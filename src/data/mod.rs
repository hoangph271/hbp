#[cfg(test)]
use self::lib::DbError;
#[cfg(test)]
use async_std::fs;

use rocket::async_trait;
use sled::Db;

pub mod lib;
pub mod models;

pub mod profile_orm;
pub mod tiny_url_orm;
pub mod user_orm;

#[async_trait]
pub trait OrmInit {
    #[cfg(test)]
    fn db_file_name(&self) -> String {
        tempfile::tempdir()
            .unwrap()
            .path()
            .to_string_lossy()
            .to_string()
    }

    #[cfg(not(test))]
    fn db_file_name(&self) -> String;

    fn get_db(&self) -> Result<Db, sled::Error> {
        sled::open(self.db_file_name())
    }

    #[cfg(test)]
    async fn init_table(&self) -> Result<(), DbError> {
        fs::create_dir_all(self.db_file_name())
            .await
            .unwrap_or_else(|e| {
                panic!("init_table() failed: {e}");
            });

        Ok(())
    }

    #[cfg(test)]
    async fn drop_table(&self) -> Result<(), DbError> {
        if async_std::path::Path::new(&self.db_file_name())
            .exists()
            .await
        {
            fs::remove_dir_all(self.db_file_name())
                .await
                .unwrap_or_else(|e| {
                    panic!("drop_table() failed: {e}");
                });
        }

        Ok(())
    }

    #[cfg(test)]
    async fn reset_table(&self) -> Result<(), DbError> {
        self.drop_table().await?;
        self.init_table().await?;

        Ok(())
    }
}
