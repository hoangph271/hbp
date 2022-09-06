use std::path::Path;

use httpstatus::StatusCode;
use image::ImageFormat;
use rocket::http::Status;
use serde::Serializer;
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
pub mod string;
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

pub fn status_code_serialize<S>(val: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(val.as_u16())
}
