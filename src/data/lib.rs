use diesel::result::Error;
use stargate_grpc::{
    result::{ColumnPositions, ResultSetMapper, TryFromRow},
    *,
};

#[derive(Debug)]
pub enum OrmError {
    NotFound,
    DieselError(Error),
}

pub mod post_orm {
    use crate::data::lib::OrmError;
    use crate::data::models::posts_model::*;
    use crate::data::schema::tbl_posts;
    use crate::data::schema::tbl_posts::dsl::*;
    use diesel::prelude::*;
    use diesel::result::Error;

    pub fn get_one(conn: &SqliteConnection, post_id: &str) -> Result<Post, OrmError> {
        match tbl_posts::table
            .filter(tbl_posts::id.eq(post_id))
            .first(conn)
        {
            Ok(post) => Ok(post),
            Err(e) => Err(OrmError::DieselError(e)),
        }
    }

    pub fn get_posts(conn: &SqliteConnection) -> Vec<Post> {
        tbl_posts.load(conn).expect("Error loading posts")
    }

    pub fn delete_one(conn: &SqliteConnection, post_id: &str) -> usize {
        diesel::delete(tbl_posts.filter(id.eq(post_id)))
            .execute(conn)
            .expect(&*format!("Error deleting post {}", post_id))
    }

    pub fn create_post(conn: &SqliteConnection, new_post: NewPost) -> Result<Post, Error> {
        diesel::insert_into(tbl_posts::table)
            .values(InsertableNewPost::from(new_post))
            .execute(conn)
            .expect("insert new_post failed");

        // FIXME: #Shame, I know
        // * It's SQLite, and I'm an idiot, I don't know how to return the just inserted record
        tbl_posts::table.order(tbl_posts::id.desc()).first(conn)
    }

    pub fn update_one(
        conn: &SqliteConnection,
        updated_post: UpdatedPost,
    ) -> Result<Post, OrmError> {
        let update_result =
            diesel::update(tbl_posts.filter(tbl_posts::id.eq(updated_post.id.clone())))
                .set((
                    tbl_posts::title.eq(updated_post.title),
                    tbl_posts::body.eq(updated_post.body),
                    tbl_posts::published.eq(updated_post.published),
                ))
                .execute(conn);

        match update_result {
            Ok(val) => {
                if val == 1 {
                    get_one(conn, &updated_post.id)
                } else {
                    Err(OrmError::NotFound)
                }
            }
            Err(e) => Err(OrmError::DieselError(e)),
        }
    }

    use super::build_stargate_client;
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
                    username text,
                    hashed_password text,
                    title text,
                    id text,
                    PRIMARY KEY (username, id)
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
                "INSERT INTO users(id, username, hashed_password, title) \
                    VALUES (:id, :username, :hashed_password, :title)",
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
pub async fn execute_stargate_query_for_one<T: ColumnPositions + TryFromRow>(
    query: stargate_grpc::Query,
) -> Option<T> {
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
