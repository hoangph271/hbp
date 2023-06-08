use async_std::fs;
use rocket::async_trait;

use self::lib::DbError;

pub mod lib;
pub mod models;

pub mod profile_orm;
pub mod tiny_url_orm;
pub mod user_orm;

#[async_trait]
pub trait OrmInit {
    fn db_file_name(&self) -> String;

    async fn init_table(&self) -> Result<(), DbError> {
        fs::File::create(self.db_file_name())
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
            fs::remove_file(self.db_file_name())
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
