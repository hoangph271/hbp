use anyhow::Result;
use rocket::tokio;

use crate::data::{lib::DbError, profile_orm::ProfileOrm, OrmInit};

fn get_profile_orm() -> ProfileOrm {
    ProfileOrm::default()
}

#[tokio::test]
async fn can_prepare_each_test() -> Result<(), DbError> {
    get_profile_orm().reset_table().await
}

#[tokio::test]
async fn can_handle_non_exist_profile() -> Result<()> {
    todo!()
    // let profile_orm = get_profile_orm();
    // profile_orm.reset_table().await?;

    // let username = String::from("username");

    // let maybe_profile = profile_orm.find_one(&username).await?;

    // assert!(maybe_profile.is_none());

    // Ok(())
}

#[tokio::test]
async fn can_create_minimal_profile() -> Result<()> {
    todo!()
    // let profile_orm = get_profile_orm();
    // profile_orm.reset_table().await?;

    // let username = "username".to_string();

    // let maybe_profile = profile_orm
    //     .create_profile(DbProfile::from_username(username.clone()))
    //     .await?;

    // assert_eq!(maybe_profile.username, username);
    // assert_eq!(maybe_profile.avatar_url, None);

    // Ok(())
}
