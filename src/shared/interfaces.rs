use httpstatus::StatusCode;
use log::error;
use okapi::openapi3::Responses;
use rocket::response::Responder;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use serde::{Serialize, Serializer};
use std::error::Error;

use crate::utils::responders::{build_json_response, HbpResponse};

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
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self {
            status_code: StatusCode::InternalServerError,
            errors: vec![format!("{e}")],
        }
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

    pub fn unprocessable_entity() -> ApiError {
        Self::from_status(StatusCode::UnprocessableEntity)
    }

    pub fn internal_server_error() -> ApiError {
        Self::from_status(StatusCode::InternalServerError)
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(build_json_response(self))
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

pub type ApiResult<T> = Result<T, ApiError>;
