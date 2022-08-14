use httpstatus::StatusCode;
use log::*;
use serde::Serialize;
use stargate_grpc::{
    result::{ColumnPositions, ResultSetMapper, TryFromRow},
    *,
};
use std::vec;
use thiserror::Error;

#[derive(Debug)]
#[allow(unused)]
pub enum OrmError {
    NotFound,
}

pub mod post_orm {
    use super::{execute_stargate_query_for_vec, stargate_client_from_env, DbError};
    use crate::data::lib::OrmError;
    use crate::data::models::posts_model::*;
    use stargate_grpc::Query;

    pub fn get_one(_post_id: &str) -> Result<Post, OrmError> {
        todo!()
    }

    pub async fn get_posts() -> Result<Vec<Post>, DbError> {
        let posts_query = Query::builder().query("SELECT * FROM posts").build();

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
        let mut client = stargate_client_from_env().await?;

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

use crate::{
    shared::interfaces::ApiErrorResponse,
    utils::env::{from_env, EnvKey},
};

#[derive(Error, Debug, Serialize)]
#[error("DbError: {status_code:?} - {message}")]
pub struct DbError {
    pub status_code: StatusCode,
    pub message: String,
}

pub type DbResult<T> = Result<T, DbError>;

impl DbError {
    pub fn internal_server_error(message: String) -> DbError {
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

pub async fn build_stargate_client(
    astra_uri: &str,
    bearer_token: &str,
) -> Result<StargateClient, DbError> {
    use std::str::FromStr;

    StargateClient::builder()
        .uri(astra_uri)
        .map_err(|_| {
            DbError::internal_server_error("build_stargate_client() failed at .uri()".to_owned())
        })?
        .auth_token(AuthToken::from_str(bearer_token).unwrap())
        .tls(Some(client::default_tls_config().unwrap()))
        .connect()
        .await
        .map_err(|e| {
            let msg = format!("build_stargate_client() failed at .connect(): {e:?}");

            DbError::internal_server_error(msg)
        })
}
pub async fn stargate_client_from_env() -> Result<StargateClient, DbError> {
    build_stargate_client(
        from_env(EnvKey::AstraUri),
        from_env(EnvKey::AstraBearerToken),
    )
    .await
}
pub async fn execute_stargate_query(
    query: stargate_grpc::Query,
) -> Result<Option<ResultSet>, DbError> {
    let mut client = stargate_client_from_env().await?;

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
    let mut client = stargate_client_from_env().await?;

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
    mut client: StargateClient,
    query: stargate_grpc::Query,
) -> Result<Option<T>, DbError>
where
    T: ColumnPositions + TryFromRow,
{
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

pub fn get_keyspace() -> &'static str {
    from_env(EnvKey::AstraKeySpace)
}
