use okapi::openapi3::OpenApi;
use rocket::{routes, Route};
use rocket_okapi::{openapi_get_routes_spec, settings::OpenApiSettings};

mod api;
mod shared;
mod ui;

use api::*;
use ui::*;

pub fn users_routes() -> Vec<Route> {
    routes![index, login, signup, post_login, post_signup]
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_post_signup, api_post_signin]
}
