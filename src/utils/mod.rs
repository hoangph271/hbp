use httpstatus::StatusCode;
use rocket::http::Status;
pub mod markdown;
pub mod responders;
pub mod setup_logger;
pub mod template;
pub mod types;
pub mod jwt;
pub mod constants;
pub mod env;

pub fn status_from(status_code: StatusCode) -> Status {
    Status::from_code(status_code.as_u16())
        .expect("status_from() failed for {status_code}")
}
