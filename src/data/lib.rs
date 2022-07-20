use std::vec;

use httpstatus::StatusCode;
use serde::Serialize;
use stargate_grpc::{
    result::{ColumnPositions, ResultSetMapper, TryFromRow},
    *,
};

#[derive(Debug)]
#[allow(unused)]
pub enum OrmError {
    NotFound,
}

pub mod post_orm {
    use super::{build_stargate_client, execute_stargate_query_for_vec, DbError};
    use crate::data::lib::OrmError;
    use crate::data::models::posts_model::*;
    use stargate_grpc::Query;

    pub fn get_one(_post_id: &str) -> Result<Post, OrmError> {
        todo!()
    }

    pub async fn get_posts() -> Result<Vec<Post>, DbError> {
        let posts_query = Query::builder()
            .keyspace("astra")
            .query("SELECT * FROM posts")
            .build();

        let val = execute_stargate_query_for_vec(posts_query).await?;

        Ok(val.unwrap_or_default())
    }

    pub fn delete_one(_post_id: &str) -> usize {
        todo!()
    }

    pub fn create_post(_new_post: NewPost) -> Result<Post, ()> {
        todo!()
    }

    pub fn update_one(_updated_post: UpdatedPost) -> Result<Post, OrmError> {
        todo!()
    }

    pub async fn init_posts_table() -> Result<(), DbError> {
        let mut client = build_stargate_client().await?;

        let create_posts_table = stargate_grpc::Query::builder()
            .query(
                "CREATE TABLE IF NOT EXISTS astra.posts \
                    (title text, body text, published Boolean, id int, PRIMARY KEY (id));",
            )
            .build();

        client.execute_query(create_posts_table).await.unwrap();

        println!("created posts table");

        Ok(())
    }
}

pub mod user_orm {
    use super::{
        build_stargate_client, execute_stargate_query, execute_stargate_query_for_one, DbError,
    };
    use crate::data::models::users_model::*;
    use httpstatus::StatusCode;
    use stargate_grpc::Query;

    pub async fn init_users_table() -> Result<(), DbError> {
        let mut client = build_stargate_client().await?;

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
                DbError::internal_server_error(
                    "init_users_table failed at .execute_query()".to_owned(),
                )
            })?;

        println!("created users table");
        Ok(())
    }

    pub async fn find_one(username: &str) -> Result<Option<User>, DbError> {
        let user_query = Query::builder()
            .keyspace("astra")
            .query("SELECT * FROM users WHERE username = :username")
            .bind_name("username", username)
            .build();

        let maybe_user: Option<User> = execute_stargate_query_for_one(user_query).await?;

        Ok(maybe_user)
    }

    pub async fn create_user(new_user: NewUser) -> Result<User, DbError> {
        let new_user: InsertableNewUser = new_user.into();

        let user_query = Query::builder()
            .keyspace("astra")
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
}

use crate::{
    shared::interfaces::ApiErrorResponse,
    utils::env::{from_env, EnvKey},
};

#[derive(Debug, Serialize)]
pub struct DbError {
    pub status_code: StatusCode,
    pub message: String,
}
impl DbError {
    fn internal_server_error(message: String) -> DbError {
        DbError {
            status_code: StatusCode::InternalServerError,
            message,
        }
    }
}
impl From<DbError> for ApiErrorResponse {
    fn from(db_error: DbError) -> Self {
        ApiErrorResponse {
            status_code: db_error.status_code,
            errors: vec![db_error.message],
        }
    }
}

pub async fn build_stargate_client() -> Result<StargateClient, DbError> {
    let astra_uri = from_env(EnvKey::AstraUri);
    let bearer_token = from_env(EnvKey::AstraBearerToken);
    use std::str::FromStr;

    let stargate_client = StargateClient::builder()
        .uri(astra_uri)
        .map_err(|_| {
            DbError::internal_server_error("build_stargate_client() failed at .uri()".to_owned())
        })?
        .auth_token(AuthToken::from_str(bearer_token).unwrap())
        .tls(Some(client::default_tls_config().unwrap()))
        .connect()
        .await
        .map_err(|_| {
            DbError::internal_server_error(
                "build_stargate_client() failed at .connect()".to_owned(),
            )
        })?;

    Ok(stargate_client)
}
pub async fn execute_stargate_query(
    query: stargate_grpc::Query,
) -> Result<Option<ResultSet>, DbError> {
    let mut client = build_stargate_client().await?;

    let response = client.execute_query(query).await.map_err(|_| {
        DbError::internal_server_error(
            "execute_stargate_query failed at .execute_query()".to_owned(),
        )
    })?;

    Ok(response.try_into().ok())
}
pub async fn execute_stargate_query_for_vec<T>(
    query: stargate_grpc::Query,
) -> Result<Option<Vec<T>>, DbError>
where
    T: ColumnPositions + TryFromRow,
{
    let mut client = build_stargate_client().await?;

    let response = client.execute_query(query).await.unwrap();

    let result_set: ResultSet = response.try_into().unwrap();

    let mapper: ResultSetMapper<T> = result_set.mapper().unwrap();

    let items: Vec<T> = result_set
        .rows
        .into_iter()
        .map(|row| {
            let item: T = mapper.try_unpack(row).unwrap();

            item
        })
        .collect();

    Ok(Some(items))
}
pub async fn execute_stargate_query_for_one<T>(
    query: stargate_grpc::Query,
) -> Result<Option<T>, DbError>
where
    T: ColumnPositions + TryFromRow,
{
    let mut client = build_stargate_client().await?;

    let response = client.execute_query(query).await.unwrap();
    let mut result_set: ResultSet = response.try_into().unwrap();

    let mapper: ResultSetMapper<T> = result_set.mapper().unwrap();

    if let Some(row) = result_set.rows.pop() {
        match mapper.try_unpack(row) {
            Ok(val) => Ok(Some(val)),
            Err(_) => Err(DbError::internal_server_error(
                "execute_stargate_query_for_one failed at .try_unpack()".to_owned(),
            )),
        }
    } else {
        Ok(None)
    }
}
