use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct HbpError {
    msg: String,
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
impl From<httpstatus::StatusCode> for HbpError {
    fn from(status_code: httpstatus::StatusCode) -> HbpError {
        HbpError::from_message(&format!("StatusCode: {}", status_code.as_u16()))
    }
}
impl From<std::io::Error> for HbpError {
    fn from(std_error: std::io::Error) -> HbpError {
        error!("{}", std_error);
        HbpError::from_message("IO Error")
    }
}
impl From<regex::Error> for HbpError {
    fn from(regex_error: regex::Error) -> HbpError {
        error!("{}", regex_error);
        HbpError::from_message("Regex Error")
    }
}
impl From<anyhow::Error> for HbpError {
    fn from(anyhow_error: anyhow::Error) -> HbpError {
        error!("{}", anyhow_error);
        HbpError::from_message("anyhow Error")
    }
}

impl HbpError {
    pub fn from_message(msg: &str) -> HbpError {
        HbpError {
            msg: String::from(msg),
        }
    }
    pub fn unimplemented() -> HbpError {
        HbpError {
            msg: String::from("unimplemented"),
        }
    }
}

pub type HbpResult<T> = Result<T, HbpError>;
