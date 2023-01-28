use std::path::{Path, MAIN_SEPARATOR};

use httpstatus::StatusCode;
use image::ImageFormat;
use rocket::http::Status;
use tempfile::NamedTempFile;

use self::responders::HbpResult;

pub mod auth;
pub mod constants;
pub mod cors;
pub mod env;
pub mod guards;
pub mod markdown;
pub mod marper;
pub mod responders;
pub mod setup_logger;
pub mod template;

pub fn timestamp_now() -> i64 {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .unwrap_or_else(|| panic!("checked_add_signed() failed"))
        .timestamp()
}

pub fn status_from(status_code: StatusCode) -> Status {
    Status::from_code(status_code.as_u16())
        .unwrap_or_else(|| panic!("status_code {} is NOT valid", status_code.as_u16()))
}

pub fn create_thumbnail(path: &Path) -> HbpResult<NamedTempFile> {
    let suffix = ImageFormat::Png.extensions_str().first().unwrap_or(&"png");

    let mut thumbnail = tempfile::Builder::new()
        .suffix(&format!(".{suffix}"))
        .tempfile()?;

    image::open(path)?
        .thumbnail(512, 512)
        .write_to(&mut thumbnail, ImageFormat::Png)?;

    Ok(thumbnail)
}

pub fn url_encode_path(path: &str) -> String {
    path.split(MAIN_SEPARATOR)
        .map(|part| urlencoding::encode(part).to_string())
        .collect::<Vec<String>>()
        .join("/")
}
