use log::*;
use std::error::Error;
use std::fmt;

use httpstatus::StatusCode::{self};
use rocket::response::Responder;

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

impl<'r> Responder<'r, 'static> for HbpError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Ok(HbpResponse::json(self.msg, Some(self.status_code)).into())
    }
}

impl HbpError {
    pub fn from_message(msg: &str, status_code: StatusCode) -> HbpError {
        HbpError {
            status_code,
            msg: msg.to_owned(),
        }
    }

    pub fn from_std_error(std_error: std::io::Error, status_code: StatusCode) -> HbpError {
        error!("{}", std_error);
        HbpError::from_message("IO Error", status_code)
    }

    pub fn not_implemented() -> HbpError {
        HbpError {
            msg: StatusCode::NotImplemented.reason_phrase().to_owned(),
            status_code: StatusCode::NotImplemented,
        }
    }

    pub fn unauthorized() -> HbpError {
        HbpError {
            msg: StatusCode::Unauthorized.reason_phrase().to_owned(),
            status_code: StatusCode::Unauthorized,
        }
    }
}

pub type HbpResult<T> = Result<T, HbpError>;
