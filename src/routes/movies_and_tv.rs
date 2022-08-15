use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use serde::Serialize;
use stargate_grpc::{Query, ResultSet};
use stargate_grpc_derive::TryFromRow;

use crate::{
    data::lib::{
        execute_stargate_query, execute_stargate_query_for_one, get_keyspace,
        stargate_client_from_env,
    },
    shared::interfaces::{ApiItemResponse, ApiListResponse},
    utils::{
        responders::HbpResponse,
        types::{HbpError, HbpResult},
    },
};

#[derive(TryFromRow, Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[openapi]
#[get("/")]
async fn api_get_shows() -> HbpResult<HbpResponse> {
    let query = Query::builder()
        .keyspace(get_keyspace())
        .query("SELECT title, show_id FROM movies_and_tv")
        .build();

    let client = stargate_client_from_env().await?;
    let result_set: ResultSet = execute_stargate_query(client, query)
        .await
        .unwrap()
        .unwrap();

    let mapper = result_set.mapper().unwrap();

    let movies_and_tv: Vec<MovieOrTv> = result_set
        .rows
        .into_iter()
        .map(|row| {
            let movie_or_tv: MovieOrTv = mapper.try_unpack(row).unwrap();

            movie_or_tv
        })
        .collect();

    Ok(ApiListResponse::ok(movies_and_tv).into())
}

#[openapi]
#[get("/<show_id>")]
async fn api_get_one(show_id: i64) -> HbpResult<HbpResponse> {
    let query = Query::builder()
        .keyspace(get_keyspace())
        .query("SELECT title, show_id FROM movies_and_tv WHERE show_id = :show_id")
        .bind_name("show_id", show_id)
        .build();

    let client = stargate_client_from_env().await?;

    execute_stargate_query_for_one::<MovieOrTv>(client, query)
        .await?
        .map(|show| ApiItemResponse::ok(show).into())
        .ok_or_else(HbpError::not_found)
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_one, api_get_shows]
}
