use httpstatus::StatusCode;
use okapi::openapi3::Responses;
use rocket::response::Responder;
use rocket_okapi::response::OpenApiResponderInner;
use serde::Serialize;

use crate::utils::{responders::HbpResponse, status_code_serialize};

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

impl<T: Serialize> From<ApiList<T>> for HbpResponse {
    fn from(api_list: ApiList<T>) -> HbpResponse {
        let status_code = api_list.status_code.clone();
        HbpResponse::json(api_list, Some(status_code)).unwrap_or_else(|e| e.into())
    }
}

impl<'r, T: Serialize> Responder<'r, 'r> for ApiList<T> {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        HbpResponse::from(self).respond_to(req)
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
