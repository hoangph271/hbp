use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use serde::Serialize;
use stargate_grpc::{Query, ResultSet};
use stargate_grpc_derive::TryFromRow;

use crate::{
    data::lib::{execute_stargate_query, execute_stargate_query_for_one},
    shared::interfaces::{ApiErrorResponse, ApiItemResponse, ApiListResponse},
    utils::responders::HbpResponse,
};

#[derive(TryFromRow, Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[openapi]
#[get("/")]
async fn api_get_shows() -> HbpResponse {
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

    ApiListResponse::ok(movies_and_tv).into()
}

#[openapi]
#[get("/<show_id>")]
async fn api_get_one(show_id: i64) -> HbpResponse {
    let query = Query::builder()
        .keyspace("astra")
        .query("SELECT title, show_id FROM movies_and_tv WHERE show_id = :show_id")
        .bind_name("show_id", show_id)
        .build();

    let maybe_show: Option<MovieOrTv> = execute_stargate_query_for_one(query).await.unwrap();

    if let Some(show) = maybe_show {
        ApiItemResponse::ok(show).into()
    } else {
        ApiErrorResponse::bad_request(vec![]).into()
    }
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_one, api_get_shows]
}
