use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use serde::Serialize;

use crate::utils::responders::HbpApiResult;

#[derive(Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[openapi]
#[get("/")]
async fn api_get_shows() -> HbpApiResult<MovieOrTv> {
    todo!()
    // let query = Query::builder()
    //     .keyspace(get_keyspace())
    //     .query("SELECT title, show_id FROM movies_and_tv")
    //     .build();

    // let client = stargate_client_from_env().await?;
    // let result_set: Option<ResultSet> = execute_stargate_query(client, query).await?;

    // let movies_and_tv: Vec<MovieOrTv> = match result_set {
    //     Some(result_set) => {
    //         let mapper = result_set.mapper().unwrap_or_else(|e| {
    //             error!("{e}");
    //             panic!("result_set.mapper() failed")
    //         });

    //         let movies_and_tv: Vec<MovieOrTv> = result_set
    //             .rows
    //             .into_iter()
    //             .filter_map(|row| {
    //                 mapper
    //                     .try_unpack(row)
    //                     .map_err(|e| {
    //                         error!("try_unpack failed: {:?}", e);
    //                         e
    //                     })
    //                     .ok()
    //             })
    //             .collect();

    //         movies_and_tv
    //     }
    //     None => vec![],
    // };

    // Ok(ApiList::ok(movies_and_tv).into())
}

#[openapi]
#[get("/<_show_id>")]
async fn api_get_one(_show_id: i64) -> HbpApiResult<MovieOrTv> {
    todo!()
    // let query = Query::builder()
    //     .keyspace(get_keyspace())
    //     .query("SELECT title, show_id FROM movies_and_tv WHERE show_id = :show_id")
    //     .bind_name("show_id", show_id)
    //     .build();

    // let client = stargate_client_from_env().await?;

    // execute_stargate_query_for_one::<MovieOrTv>(client, query)
    //     .await?
    //     .map(|show| ApiItem::ok(show).into())
    //     .ok_or_else(|| ApiError::not_found().into())
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_one, api_get_shows]
}
