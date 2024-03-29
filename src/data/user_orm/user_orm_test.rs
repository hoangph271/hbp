use anyhow::Result;
use rocket::tokio;

use crate::data::{lib::DbError, models::users_model::DbUser, OrmInit};

use super::UserOrm;

fn get_user_orm() -> UserOrm {
    UserOrm::default()
}
fn get_test_db() -> sled::Db {
    UserOrm::default().get_db().unwrap()
}

#[tokio::test]
async fn can_prepare_each_test() -> Result<(), DbError> {
    get_user_orm().reset_table().await
}

#[tokio::test]
async fn can_create_minimal_user() -> Result<()> {
    get_user_orm().reset_table().await?;

    let new_user = DbUser {
        username: "username".to_owned(),
        hashed_password: "hashed_password".to_owned(),
        title: "username".to_owned(),
    };

    let user = get_user_orm()
        .create_user(&get_test_db(), new_user.clone())
        .await?;

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(user.title, new_user.username);

    Ok(())
}
