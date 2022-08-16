use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

use crate::utils::{auth::UserPayload, responders::HbpResponse};

#[openapi]
#[get("/")]
pub async fn api_get_profile(_jwt: UserPayload) -> HbpResponse {
    todo!()
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_profile]
}
