use httpstatus::StatusCode;
use rocket::http::Status;

pub mod auth;
pub mod constants;
pub mod cors;
pub mod env;
pub mod guards;
pub mod markdown;
pub mod marper;
pub mod responders;
pub mod setup_logger;
pub mod string;
pub mod template;
pub mod types;

pub fn timestamp_now() -> i64 {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .expect("checked_add_signed() failed")
        .timestamp()
}

pub fn status_from(status_code: StatusCode) -> Status {
    Status::from_code(status_code.as_u16())
        .unwrap_or_else(|| panic!("status_code {} is NOT valid", status_code.as_u16()))
}
