use crate::utils;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel::sqlite::SqliteConnection;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

fn init_pool() -> Result<SqlitePool, PoolError> {
    let database_url = utils::env::from_env(utils::env::EnvKey::DatabaseUrl);
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    println!("Loading SQLite DB: {database_url}");

    Pool::builder().build(manager)
}

pub fn establish_connection() -> SqlitePool {
    init_pool().expect("init_pool() failed")
}
