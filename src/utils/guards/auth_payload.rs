use crate::utils::auth::{AuthPayload, UserPayload};
use crate::utils::constants;
use crate::utils::types::{HbpError, HbpResult};
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

fn get_user_jwt(req: &Request) -> Option<UserPayload> {
    match jwt_str_from_query_params(req).or_else(|| {
        req.cookies()
            .get_private(constants::cookies::USER_JWT)
            .map(|val| val.value().to_owned())
    }) {
        Some(token) => UserPayload::decode(&token).ok(),
        None => None,
    }
}
fn get_jwt(req: &Request) -> HbpResult<AuthPayload> {
    let jwt_str = jwt_str_from_query_params(req)
        .or_else(|| {
            req.cookies()
                .get_private(constants::cookies::USER_JWT)
                .map(|val| val.value().to_owned())
        })
        .or_else(|| {
            req.cookies()
                .get_private(constants::cookies::RESOURCE_JWT)
                .map(|val| val.value().to_owned())
        });

    match jwt_str {
        Some(token) => AuthPayload::decode(&token),
        None => Err(HbpError::unauthorized()),
    }
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_user_jwt(req) {
            Some(jwt) => Outcome::Success(jwt),
            None => Outcome::Failure((Status::Unauthorized, HbpError::unauthorized())),
        }
    }
}
