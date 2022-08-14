use anyhow::Result;
use rocket::tokio;

use super::UserOrm;

const TEST_KEYSPACE: &str = "astra_test";

#[tokio::test]
async fn can_int_user_test_db() -> Result<()> {
    UserOrm {
        keyspace: TEST_KEYSPACE.to_owned(),
    }
    .init_users_table()
    .await?;

    Ok(())
}
