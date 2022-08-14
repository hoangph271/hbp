use anyhow::Result;
use rocket::tokio;

use super::UserOrm;

#[tokio::test]
async fn can_int_user_test_db() -> Result<()> {
    UserOrm {
        keyspace: dotenv!("TEST_ASTRA_KEY_SPACE").to_owned(),
    }
    .init_users_table()
    .await?;

    Ok(())
}
