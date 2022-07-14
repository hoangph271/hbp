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
    use super::{build_stargate_client, execute_stargate_query_for_vec};
    use crate::data::lib::OrmError;
    use crate::data::models::posts_model::*;
    use stargate_grpc::Query;

    pub fn get_one(_post_id: &str) -> Result<Post, OrmError> {
        todo!()
    }

    pub async fn get_posts() -> Vec<Post> {
        let posts_query = Query::builder()
            .keyspace("astra")
            .query("SELECT * FROM posts")
            .build();

        execute_stargate_query_for_vec(posts_query).await.unwrap()
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

    pub async fn init_posts_table() {
        let mut client = build_stargate_client().await;

        let create_posts_table = stargate_grpc::Query::builder()
            .query(
                "CREATE TABLE IF NOT EXISTS astra.posts \
                    (title text, body text, published Boolean, id int, PRIMARY KEY (id));",
            )
            .build();

        client.execute_query(create_posts_table).await.unwrap();

        println!("created posts table");
    }
}

pub mod user_orm {
    use super::{build_stargate_client, execute_stargate_query, execute_stargate_query_for_one};
    use crate::data::models::users_model::*;
    use stargate_grpc::Query;

    pub async fn init_users_table() {
        let mut client = build_stargate_client().await;

        let create_users_table = stargate_grpc::Query::builder()
            .query(
                "CREATE TABLE IF NOT EXISTS astra.users (
                    username text PRIMARY KEY,
                    hashed_password text,
                    title text,
                )",
            )
            .build();

        client.execute_query(create_users_table).await.unwrap();

        println!("created users table");
    }

    pub async fn find_one(username: &str) -> Option<User> {
        let user_query = Query::builder()
            .keyspace("astra")
            .query("SELECT * FROM users WHERE username = :username")
            .bind_name("username", username)
            .build();

        execute_stargate_query_for_one(user_query).await
    }

    pub async fn create_user(new_user: NewUser) -> Option<User> {
        let new_user: InsertableNewUser = new_user.into();

        let user_query = Query::builder()
            .keyspace("astra")
            .query(
                "INSERT INTO users(username, hashed_password, title) \
                    VALUES (:username, :hashed_password, :title)",
            )
            .bind(new_user.clone())
            .build();

        execute_stargate_query(user_query).await;

        Some(new_user.into())
    }
}

use crate::utils::env::{from_env, EnvKey};

pub async fn build_stargate_client() -> StargateClient {
    let astra_uri = from_env(EnvKey::AstraUri);
    let bearer_token = from_env(EnvKey::AstraBearerToken);
    use std::str::FromStr;

    StargateClient::builder()
        .uri(astra_uri)
        .unwrap()
        .auth_token(AuthToken::from_str(bearer_token).unwrap())
        .tls(Some(client::default_tls_config().unwrap()))
        .connect()
        .await
        .unwrap()
}
pub async fn execute_stargate_query(query: stargate_grpc::Query) -> Option<ResultSet> {
    let mut client = build_stargate_client().await;

    let response = client.execute_query(query).await.unwrap();

    response.try_into().ok()
}
pub async fn execute_stargate_query_for_vec<T>(query: stargate_grpc::Query) -> Option<Vec<T>>
where
    T: ColumnPositions + TryFromRow,
{
    let mut client = build_stargate_client().await;

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

    Some(items)
}
pub async fn execute_stargate_query_for_one<T>(query: stargate_grpc::Query) -> Option<T>
where
    T: ColumnPositions + TryFromRow,
{
    let mut client = build_stargate_client().await;

    let response = client.execute_query(query).await.unwrap();
    let mut result_set: ResultSet = response.try_into().unwrap();

    let mapper: ResultSetMapper<T> = result_set.mapper().unwrap();

    if let Some(row) = result_set.rows.pop() {
        mapper.try_unpack(row).ok()
    } else {
        None
    }
}
