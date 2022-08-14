use anyhow::Result;
use rocket::tokio;

use super::UserOrm;

#[tokio::test]
async fn can_init_user_test_db() -> Result<()> {
    UserOrm {
        keyspace: dotenv!("TEST_ASTRA_KEY_SPACE").to_owned(),
        astra_uri: dotenv!("TEST_ASTRA_URI").to_owned(),
        bearer_token: dotenv!("TEST_ASTRA_BEARER_TOKEN").to_owned(),
    }
    .init_users_table()
    .await?;

    Ok(())
}
