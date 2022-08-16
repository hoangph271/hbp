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
        todo!()
    }

    #[cfg(test)]
    async fn drop_table(&self) -> Result<(), DbError> {
        todo!()
    }
}
