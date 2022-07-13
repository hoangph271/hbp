use crate::utils::env::{from_env, EnvKey};
use crate::utils::responders::HbpResponse;
use stargate_grpc::*;

async fn build_stargate_client() -> StargateClient {
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

use serde::Serialize;
use stargate_grpc_derive::TryFromRow;

#[derive(TryFromRow, Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[get("/")]
pub async fn get_all_shows() -> HbpResponse {
    let mut client = build_stargate_client().await;
    println!("created client {:?}", client);

    let query = Query::builder()
        .keyspace("astra")
        .query("SELECT title, show_id FROM movies_and_tv")
        .build();

    let response = client.execute_query(query).await.unwrap();
    let result_set: ResultSet = response.try_into().unwrap();

    let mapper = result_set.mapper().unwrap();

    let movies_and_tv: Vec<MovieOrTv> = result_set
        .rows
        .into_iter()
        .map(|row| {
            let movie_or_tv: MovieOrTv = mapper.try_unpack(row).unwrap();

            movie_or_tv
        })
        .collect();

    HbpResponse::json(movies_and_tv, None)
}

#[get("/<show_id>")]
pub async fn get_one_show(show_id: i64) -> HbpResponse {
    let mut client = build_stargate_client().await;
    println!("created client {:?}", client);

    let query = Query::builder()
        .keyspace("astra")
        .query("SELECT title, show_id FROM movies_and_tv WHERE show_id = :show_id")
        .bind_name("show_id", show_id)
        .build();

    let response = client.execute_query(query).await.unwrap();
    let mut result_set: ResultSet = response.try_into().unwrap();

    let mapper = result_set.mapper().unwrap();

    match result_set.rows.pop() {
        Some(row) => {
            let show: MovieOrTv = mapper.try_unpack(row).unwrap();
            HbpResponse::json(show, None)
        }
        None => HbpResponse::not_found(),
    }
}
