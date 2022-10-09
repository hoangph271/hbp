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
    use crate::data::{OrmConfig, OrmInit};
    use log::info;
    use rocket::async_trait;
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

    #[derive(Default)]
    pub struct PostOrm {
        orm_config: OrmConfig,
    }

    #[async_trait]
    impl OrmInit for PostOrm {
        fn orm_config(&self) -> &OrmConfig {
            &self.orm_config
        }

        fn table_name(&self) -> String {
            "posts".to_owned()
        }

        async fn init_table(&self) -> Result<(), DbError> {
            let mut client = stargate_client_from_env().await?;

            let create_posts_table = stargate_grpc::Query::builder()
                .query(
                    "CREATE TABLE IF NOT EXISTS astra.posts \
                        (title text, body text, published Boolean, id int, PRIMARY KEY (id));",
                )
                .build();

            client
                .execute_query(create_posts_table)
                .await
                .unwrap_or_else(|e| panic!("execute_query() failed: {:?}", e));

            info!("created posts table");

            Ok(())
        }
    }
}

use crate::{
    shared::interfaces::ApiError,
    utils::{
        env::{from_env, EnvKey},
        responders::HbpError,
    },
};

use super::OrmConfig;

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
impl From<DbError> for HbpError {
    fn from(db_error: DbError) -> Self {
        ApiError::new(db_error.status_code, vec![db_error.message]).into()
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
        .auth_token(AuthToken::from_str(bearer_token).map_err(|e| {
            error!("{e}");
            DbError::internal_server_error(format!("bearer_token invalid: {bearer_token}"))
        })?)
        .tls(Some(
            client::default_tls_config().unwrap_or_else(|_| panic!("build default client fail")),
        ))
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
pub async fn stargate_client_from(orm_config: &OrmConfig) -> Result<StargateClient, DbError> {
    build_stargate_client(&orm_config.astra_uri, &orm_config.bearer_token).await
}
pub async fn execute_stargate_query(
    mut client: StargateClient,
    query: stargate_grpc::Query,
) -> Result<Option<ResultSet>, DbError> {
    let response = client.execute_query(query).await.map_err(|e| {
        let msg = format!("execute_stargate_query failed at .execute_query(): {e:?}");

        DbError::internal_server_error(msg)
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

    let response = client
        .execute_query(query)
        .await
        .unwrap_or_else(|e| panic!("execute_query failed: {e}"));

    let result_set: ResultSet = response
        .try_into()
        .unwrap_or_else(|e| panic!("response.try_into() failed: {e}"));

    let mapper: ResultSetMapper<T> = result_set
        .mapper()
        .unwrap_or_else(|e| panic!("mapper() failed: {e}"));

    let items: Vec<T> = result_set
        .rows
        .into_iter()
        .filter_map(|row| {
            mapper
                .try_unpack(row)
                .map_err(|e| {
                    error!("try_unpacked() failed: {e:?}");
                    e
                })
                .ok()
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
    let response = client.execute_query(query).await.map_err(|e| {
        let message = format!("execute_query() failed: {e}");
        error!("{message}");

        DbError::internal_server_error(message)
    })?;
    let mut result_set: ResultSet = response.try_into().unwrap_or_else(|e| {
        panic!("response.try_into() failed: {e}");
    });

    let mapper: ResultSetMapper<T> = result_set
        .mapper()
        .unwrap_or_else(|e| panic!("mapper() failed: {e}"));

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
