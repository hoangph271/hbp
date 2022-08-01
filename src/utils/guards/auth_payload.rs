use crate::utils::auth::{jwt, AuthPayload};
use crate::utils::constants;
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_str_from_query_params(req: &Request) -> Option<String> {
    match req.query_value::<&str>("jwt") {
        Some(jwt) => jwt.map(|val| val.to_owned()).ok(),
        None => req
            .headers()
            .get_one(constants::headers::AUTHORIZATION)
            .map(|jwt_str| jwt_str.trim()["Bearer ".len()..].to_owned()),
    }
}

pub const RESOURCE_JWT_COOKIE: &str = "resource-jwt";
pub const USER_JWT_COOKIE: &str = "user-jwt";

fn get_jwt(req: &Request) -> HbpResult<AuthPayload> {
    let jwt_str = jwt_str_from_query_params(req).or_else(|| {
        let jwt_from_cookies = req
            .cookies()
            .get_private(USER_JWT_COOKIE)
            .or_else(|| req.cookies().get_private(RESOURCE_JWT_COOKIE));

        jwt_from_cookies.map(|cookies| cookies.value().to_owned())
    });

    jwt_str
        .map(|jwt_str| jwt::verify_jwt(&jwt_str))
        .ok_or_else(|| HbpError::from_message("No valid jwt found", StatusCode::Unauthorized))?
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_jwt(req) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(e) => Outcome::Failure((Status::from_code(e.status_code.as_u16()).unwrap(), e)),
        }
    }
}
