use crate::utils::types::HbpError;
use httpstatus::StatusCode;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

pub struct Referer(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Referer {
    type Error = HbpError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Referer");
        match token {
            Some(token) => Outcome::Success(Referer(token.to_string())),
            None => Outcome::Failure((
                Status::from_code(StatusCode::Unauthorized.as_u16()).unwrap(),
                HbpError::from_message("No valid jwt found", StatusCode::Unauthorized),
            )),
        }
    }
}
