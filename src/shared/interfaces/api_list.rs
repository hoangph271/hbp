use httpstatus::StatusCode;
use okapi::openapi3::Responses;
use rocket::response::Responder;
use rocket_okapi::response::OpenApiResponderInner;
use serde::Serialize;

use crate::utils::responders::build_json_response;

use super::utils::status_code_serialize;

#[derive(Serialize)]
pub struct ApiList<T: Serialize> {
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    items: Vec<T>,
}

impl<T: Serialize> ApiList<T> {
    pub fn ok(items: Vec<T>) -> ApiList<T> {
        ApiList {
            status_code: StatusCode::Ok,
            items,
        }
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiList<T> {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(build_json_response(self))
    }
}
impl<T: Serialize> OpenApiResponderInner for ApiList<T> {
    fn responses(
        _: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<okapi::openapi3::Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}
