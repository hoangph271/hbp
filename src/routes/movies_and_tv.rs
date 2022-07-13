use serde::Serialize;
use stargate_grpc::{Query, ResultSet};
use stargate_grpc_derive::TryFromRow;

use crate::{data::lib::build_stargate_client, utils::responders::HbpResponse};

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
