use log::*;
use okapi::openapi3::Responses;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use std::error::Error;
use std::fmt;

use httpstatus::StatusCode::{self};
use rocket::response::Responder;

use crate::data::lib::DbError;

use super::responders::HbpResponse;

#[derive(Debug)]
pub struct HbpError {
    pub msg: String,
    pub status_code: StatusCode,
}

impl fmt::Display for HbpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}
impl Error for HbpError {
    fn description(&self) -> &str {
        &self.msg
    }
}
impl From<StatusCode> for HbpError {
    fn from(status_code: StatusCode) -> HbpError {
        HbpError::from_message(
            &format!("StatusCode: {}", status_code.as_u16()),
            status_code,
        )
    }
}
impl From<reqwest::Error> for HbpError {
    fn from(e: reqwest::Error) -> Self {
        error!("[reqwest::Error]: {e}");

        let msg = match e.source() {
            Some(source) => format!("{:?}", source),
            None => "Unknown error".to_owned(),
        };

        HbpError::from_message(
            &msg,
            if let Some(status_code) = e.status() {
                status_code.as_u16().into()
            } else {
                StatusCode::InternalServerError
            },
        )
    }
}
impl From<mustache::Error> for HbpError {
    fn from(e: mustache::Error) -> Self {
        HbpError {
            msg: e.to_string(),
            status_code: match e {
                mustache::Error::InvalidStr => StatusCode::UnprocessableEntity,
                mustache::Error::NoFilename => StatusCode::NotFound,
                _ => StatusCode::InternalServerError,
            },
        }
    }
}
impl From<std::str::Utf8Error> for HbpError {
    fn from(e: std::str::Utf8Error) -> Self {
        HbpError::from_message(
            &format!("UTF8 Issue: , {e}"),
            StatusCode::InternalServerError,
        )
    }
}
impl From<DbError> for HbpError {
    fn from(e: DbError) -> Self {
        HbpError {
            msg: e.message,
            status_code: e.status_code,
        }
    }
}

impl<'r> Responder<'r, 'static> for HbpError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(HbpResponse::json(self.msg, Some(self.status_code)).into())
    }
}

impl OpenApiResponderInner for HbpError {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}

impl HbpError {
    pub fn from_status(status_code: StatusCode) -> Self {
        Self {
            msg: status_code.reason_phrase().to_owned(),
            status_code,
        }
    }
    pub fn from_message(msg: &str, status_code: StatusCode) -> HbpError {
        HbpError {
            status_code,
            msg: msg.to_owned(),
        }
    }

    pub fn from_io_error(std_error: std::io::Error, status_code: StatusCode) -> HbpError {
        error!("{}", std_error);
        HbpError::from_message("IO Error", status_code)
    }

    pub fn not_implemented() -> HbpError {
        Self::from_status(StatusCode::NotImplemented)
    }

    pub fn unauthorized() -> HbpError {
        Self::from_status(StatusCode::Unauthorized)
    }

    pub fn forbidden() -> HbpError {
        Self::from_status(StatusCode::Forbidden)
    }

    pub fn not_found() -> HbpError {
        Self::from_status(StatusCode::NotFound)
    }

    pub fn bad_request(msg: String) -> HbpError {
        Self {
            msg,
            status_code: StatusCode::BadRequest,
        }
    }

    pub fn internal_server_error() -> HbpError {
        Self::from_status(StatusCode::InternalServerError)
    }
}

pub type HbpResult<T> = Result<T, HbpError>;
