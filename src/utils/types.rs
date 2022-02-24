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
impl HbpError {
    pub fn from_message(msg: &str) -> HbpError {
        HbpError {
            msg: String::from(msg),
        }
    }
}
