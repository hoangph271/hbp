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
        .unwrap()
        .timestamp()
}
