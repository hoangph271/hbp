use crate::data::{lib::*, models::users_model::*};
use httpstatus::StatusCode;
use stargate_grpc::Query;

pub async fn init_users_table() -> Result<(), DbError> {
    let mut client = stargate_client_from_env().await?;

    let create_users_table = stargate_grpc::Query::builder()
        .query(
            "CREATE TABLE IF NOT EXISTS astra.users (
                    username text PRIMARY KEY,
                    hashed_password text,
                    title text,
                )",
        )
        .build();

    client
        .execute_query(create_users_table)
        .await
        .map_err(|_| {
            DbError::internal_server_error("init_users_table failed at .execute_query()".to_owned())
        })?;

    println!("created users table");
    Ok(())
}

pub async fn find_one(username: &str) -> Result<Option<User>, DbError> {
    let user_query = Query::builder()
        .keyspace(get_keyspace())
        .query("SELECT * FROM users WHERE username = :username")
        .bind_name("username", username)
        .build();

    let maybe_user: Option<User> = execute_stargate_query_for_one(user_query).await?;

    Ok(maybe_user)
}

pub async fn create_user(new_user: NewUser) -> Result<User, DbError> {
    let new_user: InsertableNewUser = new_user.into();

    let user_query = Query::builder()
        .keyspace(get_keyspace())
        .query(
            "INSERT INTO users(username, hashed_password, title) \
                    VALUES (:username, :hashed_password, :title) \
                    IF NOT EXISTS",
        )
        .bind(new_user.clone())
        .build();

    let mut result_set = execute_stargate_query(user_query).await?.unwrap();
    let mut row = result_set.rows.pop().unwrap();
    let inserted: bool = row.try_take(0).unwrap();

    if inserted {
        Ok(new_user.into())
    } else {
        Err(DbError {
            status_code: StatusCode::Conflict,
            message: format!("username `{}` existed", new_user.username),
        })
    }
}
