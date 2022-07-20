use crate::utils::responders::HbpResponse;
use httpstatus::StatusCode;
use rocket::{http::Status, Catcher, Request};

#[catch(default)]
fn default(status: Status, _req: &Request) -> HbpResponse {
    HbpResponse::status(StatusCode::from(status.code))
}

pub fn catchers() -> Vec<Catcher> {
    catchers![default]
}
