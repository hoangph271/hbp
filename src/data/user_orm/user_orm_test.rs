use anyhow::Result;
use rocket::tokio;

use crate::data::models::users_model::NewUser;

use super::UserOrm;

fn create_user_orm() -> UserOrm {
    UserOrm {
        keyspace: dotenv!("TEST_ASTRA_KEY_SPACE").to_owned(),
        astra_uri: dotenv!("TEST_ASTRA_URI").to_owned(),
        bearer_token: dotenv!("TEST_ASTRA_BEARER_TOKEN").to_owned(),
    }
}
async fn reset_users_table() -> Result<()> {
    let user_orm = create_user_orm();

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
        avatar_url: None,
        title: None,
    };

    let user = create_user_orm().create_user(new_user.clone()).await?;

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(user.title, new_user.username);
    assert_eq!(user.avatar_url, None);

    Ok(())
}

#[tokio::test]
async fn create_user_with_avatar() -> Result<()> {
    reset_users_table().await?;

    let new_user = NewUser {
        username: "username".to_owned(),
        hashed_password: "hashed_password".to_owned(),
        avatar_url: Some("avatar.url".to_owned()),
        title: Some("title".to_owned()),
    };

    let user = create_user_orm().create_user(new_user.clone()).await?;

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(Some(user.title), new_user.title);
    assert_eq!(user.avatar_url, new_user.avatar_url);

    Ok(())
}
