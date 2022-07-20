use httpstatus::StatusCode;
use serde::{Serialize, Serializer};

use crate::utils::responders::HbpResponse;

fn status_code_serialize<S>(val: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(val.as_u16())
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    // TODO: Should be number
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    errors: Vec<String>,
}
impl From<ApiErrorResponse> for HbpResponse {
    fn from(api_error_response: ApiErrorResponse) -> HbpResponse {
        let status_code = api_error_response.status_code.clone();
        HbpResponse::json(api_error_response, Some(status_code))
    }
}
impl ApiErrorResponse {
    pub fn bad_request(errors: Vec<String>) -> ApiErrorResponse {
        ApiErrorResponse {
            status_code: StatusCode::BadRequest,
            errors,
        }
    }
}

#[derive(Serialize)]
pub struct ApiItemResponse<T>
where
    T: Serialize,
{
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

pub struct ApiListResponse<T>
where
    T: Serialize,
{
    status_code: Option<StatusCode>,
    items: Vec<T>,
}
impl<T> From<ApiListResponse<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiListResponse<T>) -> HbpResponse {
        HbpResponse::json(api_item_response.items, api_item_response.status_code)
    }
}
impl<T> ApiListResponse<T>
where
    T: Serialize,
{
    pub fn from_items(items: Vec<T>) -> ApiListResponse<T> {
        ApiListResponse {
            status_code: None,
            items,
        }
    }
}
