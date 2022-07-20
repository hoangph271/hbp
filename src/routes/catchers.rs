use crate::utils::responders::HbpResponse;
use httpstatus::StatusCode;
use rocket::{catch, http::Status, Catcher, Request, catchers};

#[catch(default)]
fn default(status: Status, _req: &Request) -> HbpResponse {
    HbpResponse::status(StatusCode::from(status.code))
}

pub fn catchers() -> Vec<Catcher> {
    catchers![default]
}
