use anyhow::Result;
use rocket::tokio;

use crate::data::models::users_model::NewUser;

use super::UserOrm;

async fn reset_users_table() -> Result<()> {
    let user_orm = UserOrm::from_test_env();

    user_orm.drop_users_table().await?;
    user_orm.init_users_table().await?;

    Ok(())
}

#[tokio::test]
async fn can_prepare_each_test() -> Result<()> {
    reset_users_table().await
}

#[tokio::test]
async fn can_create_minimal_user() -> Result<()> {
    reset_users_table().await?;

    let new_user = NewUser {
        username: "username".to_owned(),
        hashed_password: "hashed_password".to_owned(),
        title: None,
    };

    let user = UserOrm::from_test_env()
        .create_user(new_user.clone())
        .await?;

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(user.title, new_user.username);

    Ok(())
}
