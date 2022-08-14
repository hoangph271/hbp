use httpstatus::StatusCode;
use serde::{Serialize, Serializer};

use crate::utils::{responders::HbpResponse, types::HbpError};

fn status_code_serialize<S>(val: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(val.as_u16())
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub errors: Vec<String>,
}

impl From<ApiErrorResponse> for HbpResponse {
    fn from(api_error_response: ApiErrorResponse) -> HbpResponse {
        let status_code = api_error_response.status_code.clone();
        HbpResponse::json(api_error_response, Some(status_code))
    }
}
impl From<ApiErrorResponse> for HbpError {
    fn from(api_error_response: ApiErrorResponse) -> HbpError {
        HbpError {
            msg: api_error_response.errors.join(";"),
            status_code: api_error_response.status_code,
        }
    }
}
impl ApiErrorResponse {
    pub fn bad_request(errors: Vec<String>) -> ApiErrorResponse {
        ApiErrorResponse {
            status_code: StatusCode::BadRequest,
            errors,
        }
    }

    pub fn from_status(status_code: StatusCode) -> Self {
        Self {
            status_code: status_code.clone(),
            errors: vec![status_code.reason_phrase().to_string()],
        }
    }

    pub fn unauthorized() -> ApiErrorResponse {
        Self::from_status(StatusCode::Unauthorized)
    }

    pub fn forbidden() -> ApiErrorResponse {
        Self::from_status(StatusCode::Forbidden)
    }

    pub fn internal_server_error() -> ApiErrorResponse {
        Self::from_status(StatusCode::InternalServerError)
    }
}

#[derive(Serialize)]
pub struct ApiItemResponse<T>
where
    T: Serialize,
{
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    item: T,
}

impl<T> From<ApiItemResponse<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiItemResponse<T>) -> HbpResponse {
        let status_code = api_item_response.status_code.clone();
        HbpResponse::json(api_item_response, Some(status_code))
    }
}
impl<T> ApiItemResponse<T>
where
    T: Serialize,
{
    pub fn ok(item: T) -> ApiItemResponse<T> {
        ApiItemResponse {
            status_code: StatusCode::Ok,
            item,
        }
    }
}

#[derive(Serialize)]
pub struct ApiListResponse<T>
where
    T: Serialize,
{
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    items: Vec<T>,
}

impl<T> From<ApiListResponse<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiListResponse<T>) -> HbpResponse {
        let status_code = api_item_response.status_code.clone();
        HbpResponse::json(api_item_response, Some(status_code))
    }
}
impl<T> ApiListResponse<T>
where
    T: Serialize,
{
    pub fn ok(items: Vec<T>) -> ApiListResponse<T> {
        ApiListResponse {
            status_code: StatusCode::Ok,
            items,
        }
    }
}
