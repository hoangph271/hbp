use async_std::{fs};
use rocket::async_trait;

use self::lib::DbError;

pub mod lib;
pub mod models;

pub mod profile_orm;
pub mod user_orm;


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
