use rocket::{
    get,
    http::uri::{Origin, Uri},
    routes, Route, State,
};
use sled::Db;

use crate::{
    data::tiny_url_orm::TinyUrlOrm,
    utils::responders::{HbpResponse, HbpResult},
};

#[get("/<slug>")]
async fn tiny_url_slug(slug: String, db: &State<Db>) -> HbpResult<HbpResponse> {
    let tiny_url = TinyUrlOrm::default()
        .find_one(db, &slug)
        .await
        .expect("find_one() TinyUrl failed...!");

    let response = if let Some(tiny_url) = tiny_url {
        if let Ok(uri) = Uri::parse::<Origin>(&tiny_url.full_url) {
            HbpResponse::redirect(uri.origin().unwrap().to_owned())
        } else {
            HbpResponse::internal_server_error()
        }
    } else {
        HbpResponse::not_found()
    };

    Ok(response)
}

pub fn tiny_urls_routes() -> Vec<Route> {
    routes![tiny_url_slug]
}
