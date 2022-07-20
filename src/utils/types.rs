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

    pub fn unimplemented() -> HbpError {
        HbpError {
            msg: String::from("unimplemented"),
            status_code: StatusCode::InternalServerError,
        }
    }
}

pub type HbpResult<T> = Result<T, HbpError>;
