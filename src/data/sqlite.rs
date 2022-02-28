use crate::utils;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub fn establish_connection() -> SqliteConnection {
    let database_url = utils::env::from_env(utils::env::EnvKey::DatabaseUrl);
    println!("Loading SQLite DB: {database_url}");

    SqliteConnection::establish(database_url).expect(&*format!("{database_url} can NOT be opened"))
}
