use crate::shared::interfaces::ApiError;

// FIXME: Remove this HbpResult
pub type HbpResult<T> = Result<T, ApiError>;
