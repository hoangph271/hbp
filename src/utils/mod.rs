use std::path::Path;

use httpstatus::StatusCode;
use image::{ImageError, ImageFormat};
use log::error;
use rocket::http::Status;
use tempfile::NamedTempFile;

use crate::shared::interfaces::{ApiError, ApiResult};

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

impl From<ImageError> for ApiError {
    fn from(e: ImageError) -> Self {
        error!("ImageError: {e}");

        match e {
            ImageError::Decoding(e) => {
                ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
            }
            ImageError::Encoding(e) => {
                ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
            }
            ImageError::Parameter(e) => {
                ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
            }
            ImageError::Limits(e) => {
                ApiError::from_message(&e.to_string(), StatusCode::UnprocessableEntity)
            }
            ImageError::Unsupported(e) => {
                ApiError::from_message(&e.to_string(), StatusCode::UnprocessableEntity)
            }
            ImageError::IoError(e) => {
                ApiError::from_message(&format!("{e}"), StatusCode::InternalServerError)
            }
        }
    }
}
pub fn create_thumbnail(path: &Path) -> ApiResult<NamedTempFile> {
    let suffix = ImageFormat::Png.extensions_str().first().unwrap_or(&"png");

    let mut thumbnail = tempfile::Builder::new()
        .suffix(&format!(".{suffix}"))
        .tempfile()?;

    image::open(path)?
        .thumbnail(512, 512)
        .write_to(&mut thumbnail, ImageFormat::Png)?;

    Ok(thumbnail)
}
