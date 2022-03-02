use rocket_sync_db_pools::{diesel, database};

#[database("sqlite_db")]
pub struct DbConn(diesel::SqliteConnection);
