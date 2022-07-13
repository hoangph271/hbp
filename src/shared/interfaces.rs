use httpstatus::StatusCode;
use serde::Serialize;

use crate::utils::responders::HbpResponse;

pub struct ApiErrorResponse {
    status_code: Option<StatusCode>,
    errors: Vec<String>,
}
impl From<ApiErrorResponse> for HbpResponse {
    fn from(api_error_response: ApiErrorResponse) -> HbpResponse {
        HbpResponse::json(api_error_response.errors, api_error_response.status_code)
    }
}

pub struct ApiItemResponse<T>
where
    T: Serialize,
{
    status_code: Option<StatusCode>,
    item: T,
}
impl<T> From<ApiItemResponse<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiItemResponse<T>) -> HbpResponse {
        HbpResponse::json(api_item_response.item, api_item_response.status_code)
    }
}
impl<T> ApiItemResponse<T>
where
    T: Serialize,
{
    pub fn from_item(item: T) -> ApiItemResponse<T> {
        ApiItemResponse {
            status_code: None,
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
