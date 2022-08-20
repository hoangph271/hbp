use httpstatus::StatusCode;
use log::error;
use okapi::openapi3::Responses;
use rocket::response::Responder;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use serde::{Serialize, Serializer};
use std::error::Error;

use crate::utils::responders::HbpResponse;

fn status_code_serialize<S>(val: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(val.as_u16())
}

#[derive(Serialize, Clone, Debug)]
pub struct ApiError {
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub errors: Vec<String>,
}

impl From<ApiError> for HbpResponse {
    fn from(api_error_response: ApiError) -> HbpResponse {
        let status_code = api_error_response.status_code.clone();
        HbpResponse::json(api_error_response, Some(status_code))
    }
}
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        error!("[reqwest::Error]: {e}");

        let msg = match e.source() {
            Some(source) => format!("{:?}", source),
            None => "Unknown error".to_owned(),
        };

        ApiError::from_message(
            &msg,
            if let Some(status_code) = e.status() {
                status_code.as_u16().into()
            } else {
                StatusCode::InternalServerError
            },
        )
    }
}

impl ApiError {
    pub fn bad_request(errors: Vec<String>) -> ApiError {
        ApiError {
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

    pub fn from_message(msg: &str, status_code: StatusCode) -> ApiError {
        ApiError {
            status_code,
            errors: vec![msg.to_owned()],
        }
    }

    pub fn from_io_error(std_error: std::io::Error, status_code: StatusCode) -> ApiError {
        error!("{}", std_error);
        ApiError::from_message("IO Error", status_code)
    }

    pub fn unauthorized() -> ApiError {
        Self::from_status(StatusCode::Unauthorized)
    }

    pub fn not_implemented() -> ApiError {
        Self::from_status(StatusCode::NotImplemented)
    }

    pub fn not_found() -> ApiError {
        Self::from_status(StatusCode::NotFound)
    }

    pub fn forbidden() -> ApiError {
        Self::from_status(StatusCode::Forbidden)
    }

    pub fn internal_server_error() -> ApiError {
        Self::from_status(StatusCode::InternalServerError)
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(HbpResponse::json(self.clone(), Some(self.status_code)).into())
    }
}
impl OpenApiResponderInner for ApiError {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}

#[derive(Serialize)]
pub struct ApiItem<T>
where
    T: Serialize,
{
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    item: T,
}

impl<'r, T> Responder<'r, 'static> for ApiItem<T>
where
    T: Serialize,
{
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(HbpResponse::from(self).into())
    }
}
impl<T> OpenApiResponderInner for ApiItem<T>
where
    T: Serialize,
{
    fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}

impl<T> From<ApiItem<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiItem<T>) -> HbpResponse {
        let status_code = api_item_response.status_code.clone();
        HbpResponse::json(api_item_response, Some(status_code))
    }
}
impl<T> ApiItem<T>
where
    T: Serialize,
{
    pub fn ok(item: T) -> ApiItem<T> {
        ApiItem {
            status_code: StatusCode::Ok,
            item,
        }
    }
}

#[derive(Serialize)]
pub struct ApiList<T>
where
    T: Serialize,
{
    #[serde(serialize_with = "status_code_serialize")]
    status_code: StatusCode,
    items: Vec<T>,
}

impl<T> From<ApiList<T>> for HbpResponse
where
    T: Serialize,
{
    fn from(api_item_response: ApiList<T>) -> HbpResponse {
        let status_code = api_item_response.status_code.clone();
        HbpResponse::json(api_item_response, Some(status_code))
    }
}
impl<T> ApiList<T>
where
    T: Serialize,
{
    pub fn ok(items: Vec<T>) -> ApiList<T> {
        ApiList {
            status_code: StatusCode::Ok,
            items,
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
