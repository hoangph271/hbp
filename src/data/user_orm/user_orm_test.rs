use anyhow::Result;
use rocket::tokio;

use crate::data::models::users_model::NewUser;

use super::UserOrm;

fn get_user_orm() -> UserOrm {
    UserOrm {
        keyspace: dotenv!("TEST_ASTRA_KEY_SPACE").to_owned(),
        astra_uri: dotenv!("TEST_ASTRA_URI").to_owned(),
        bearer_token: dotenv!("TEST_ASTRA_BEARER_TOKEN").to_owned(),
    }
}
async fn reset_users_table() -> Result<()> {
    let user_orm = get_user_orm();

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

    get_user_orm()
        .create_user(NewUser {
            username: "username".to_owned(),
            hashed_password: "password".to_owned(),
            title: None,
        })
        .await?;

    Ok(())
}
