use log::error;
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
    let result_set: Option<ResultSet> = execute_stargate_query(client, query).await?;

    let movies_and_tv: Vec<MovieOrTv> = match result_set {
        Some(result_set) => {
            let mapper = result_set.mapper().unwrap_or_else(|e| {
                error!("{e}");
                panic!("result_set.mapper() failed")
            });

            let movies_and_tv: Vec<MovieOrTv> = result_set
                .rows
                .into_iter()
                .filter_map(|row| {
                    mapper
                        .try_unpack(row)
                        .map_err(|e| {
                            error!("try_unpack failed: {:?}", e);
                            e
                        })
                        .ok()
                })
                .collect();

            movies_and_tv
        }
        None => vec![],
    };

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
