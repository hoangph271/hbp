use anyhow::Result;
use rocket::tokio;

use crate::data::{lib::DbError, OrmConfig, OrmInit};

use super::ProfileOrm;

fn get_profile_orm() -> ProfileOrm {
    ProfileOrm {
        orm_config: OrmConfig::from_test_env(),
    }
}

#[tokio::test]
async fn can_prepare_each_test() -> Result<(), DbError> {
    get_profile_orm().reset_table().await
}

#[tokio::test]
async fn can_create_minimal_profile() -> Result<()> {
    let profile_orm = get_profile_orm();
    profile_orm.reset_table().await?;

    let username = String::from("username");

    profile_orm.find_one(&username).await?;

    Ok(())
}
