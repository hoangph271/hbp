use crate::{shared::interfaces::ApiErrorResponse, utils::responders::HbpResponse};
use httpstatus::StatusCode;
use rocket::{catch, catchers, http::Status, Catcher, Request};

#[catch(default)]
fn default(status: Status, req: &Request) -> HbpResponse {
    let path = req.uri().path();
    let is_api = path.starts_with("/api/");
    let status_code = StatusCode::from(status.code);

    if !is_api {
        return HbpResponse::status(StatusCode::from(status_code));
    }

    match status_code {
        StatusCode::NotFound => ApiErrorResponse {
            status_code: status_code.clone(),
            errors: vec![format!(
                "{} - Most likely the api endpoint does NOT exist",
                status_code.reason_phrase()
            )],
        }
        .into(),
        _ => ApiErrorResponse::from_status(status_code).into(),
    }
}

pub fn catchers() -> Vec<Catcher> {
    catchers![default]
}
