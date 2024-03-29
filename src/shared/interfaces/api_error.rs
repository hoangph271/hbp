use httpstatus::StatusCode;
use log::error;
use okapi::openapi3::Responses;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use serde::Serialize;
use std::error::Error;

use crate::utils::{responders::HbpResponse, status_code_serialize};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub errors: Vec<String>,
    #[serde(skip_serializing)]
    with_ui: bool,
}

impl ApiError {
    pub fn new(status_code: StatusCode, errors: Vec<String>) -> Self {
        Self {
            status_code,
            errors,
            with_ui: false,
        }
    }

    pub fn bad_request(errors: Vec<String>) -> Self {
        ApiError {
            status_code: StatusCode::BadRequest,
            errors,
            with_ui: false,
        }
    }

    pub fn from_status(status_code: StatusCode) -> Self {
        Self {
            with_ui: false,
            status_code: status_code.clone(),
            errors: vec![status_code.reason_phrase().to_string()],
        }
    }

    pub fn from_message(msg: &str, status_code: StatusCode) -> ApiError {
        ApiError {
            with_ui: false,
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

    pub fn append_error(mut self, error: String) -> Self {
        self.errors.push(error);

        self
    }

    pub fn with_ui(mut self) -> Self {
        self.with_ui = true;
        self
    }
}

impl From<ApiError> for HbpResponse {
    fn from(error: ApiError) -> HbpResponse {
        let status_code = error.status_code.clone();

        if error.with_ui {
            HbpResponse::from_error_status(status_code)
        } else {
            match HbpResponse::json(error, Some(status_code)) {
                Ok(json) => json,
                Err(e) => e.into(),
            }
        }
    }
}

impl<'r> rocket::response::Responder<'r, 'r> for ApiError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        HbpResponse::from(self).respond_to(req)
    }
}

impl OpenApiResponderInner for ApiError {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}
