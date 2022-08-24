use httpstatus::StatusCode;
use okapi::openapi3::Responses;
use rocket::response::Responder;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use serde::Serialize;

use crate::utils::responders::{build_json_response, HbpResponse};

use super::utils::status_code_serialize;

#[derive(Serialize)]
pub struct ApiItem<T: Serialize> {
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    item: T,
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiItem<T> {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(build_json_response(self))
    }
}
impl<T: Serialize> OpenApiResponderInner for ApiItem<T> {
    fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}
impl<T: Serialize> From<ApiItem<T>> for HbpResponse {
    fn from(item: ApiItem<T>) -> HbpResponse {
        let status_code = item.status_code.clone();
        HbpResponse::json(item, Some(status_code))
    }
}
impl<T: Serialize> ApiItem<T> {
    pub fn ok(item: T) -> ApiItem<T> {
        ApiItem {
            status_code: StatusCode::Ok,
            item,
        }
    }
}
