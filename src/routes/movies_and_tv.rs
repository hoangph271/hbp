use rocket::{get, Route, routes};
use serde::Serialize;

use crate::utils::responders::HbpApiResult;

#[derive(Serialize)]
struct MovieOrTv {
    title: String,
    show_id: i64,
}

#[get("/")]
async fn api_get_shows() -> HbpApiResult<MovieOrTv> {
    todo!()
}

#[get("/<_show_id>")]
async fn api_get_one(_show_id: i64) -> HbpApiResult<MovieOrTv> {
    todo!()
}

pub fn movies_and_tv_api_routes() -> Vec<Route> {
    routes![api_get_one, api_get_shows]
}
