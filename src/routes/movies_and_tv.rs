use serde::Serialize;
use stargate_grpc::{Query, ResultSet};
use stargate_grpc_derive::TryFromRow;

use crate::{
    data::lib::{execute_stargate_query, execute_stargate_query_for_one},
    shared::interfaces::{ApiItemResponse, ApiListResponse},
    utils::responders::HbpResponse,
};

#[derive(TryFromRow, Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[get("/")]
pub async fn get_all_shows() -> HbpResponse {
    let query = Query::builder()
        .keyspace("astra")
        .query("SELECT title, show_id FROM movies_and_tv")
        .build();

    let result_set: ResultSet = execute_stargate_query(query).await.unwrap().unwrap();

    let mapper = result_set.mapper().unwrap();

    let movies_and_tv: Vec<MovieOrTv> = result_set
        .rows
        .into_iter()
        .map(|row| {
            let movie_or_tv: MovieOrTv = mapper.try_unpack(row).unwrap();

            movie_or_tv
        })
        .collect();

    ApiListResponse::from_items(movies_and_tv).into()
}

#[get("/<show_id>")]
pub async fn get_one_show(show_id: i64) -> HbpResponse {
    let query = Query::builder()
        .keyspace("astra")
        .query("SELECT title, show_id FROM movies_and_tv WHERE show_id = :show_id")
        .bind_name("show_id", show_id)
        .build();

    let maybe_show: Option<MovieOrTv> = execute_stargate_query_for_one(query).await.unwrap();

    if let Some(show) = maybe_show {
        ApiItemResponse::from_item(show).into()
    } else {
        HbpResponse::not_found()
    }
}
