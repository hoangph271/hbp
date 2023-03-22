use crate::{shared::interfaces::ApiError, utils::responders::HbpResponse};
use httpstatus::StatusCode::{self, *};
use rocket::{catch, catchers, http::Status, Catcher, Request};

#[catch(default)]
fn default(status: Status, req: &Request) -> HbpResponse {
    let path = req.uri().path().to_string();
    let status_code = StatusCode::from(status.code);

    let is_api = path.starts_with("/api/");

    if !is_api {
        return match status_code {
            StatusCode::Unauthorized => HbpResponse::unauthorized(Some(path)),
            _ => HbpResponse::from_error_status(status_code),
        };
    }

    match status_code {
        NotFound => ApiError::from_status(status_code.clone())
            .append_error(format!(
                "{} - Most likely the api endpoint does NOT exist",
                status_code.reason_phrase()
            ))
            .into(),
        InternalServerError => ApiError::from_status(status_code.clone())
            .append_error(format!(
                "{} - Something went wrong on own end",
                status_code.reason_phrase()
            ))
            .into(),
        _ => ApiError::from_status(status_code).into(),
    }
}

pub fn catchers() -> Vec<Catcher> {
    catchers![default]
}
