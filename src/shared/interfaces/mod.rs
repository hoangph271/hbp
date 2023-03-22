mod hbp_types;
pub use hbp_types::*;

pub type ApiResult<T> = Result<T, ApiError>;
