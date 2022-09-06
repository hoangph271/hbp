use crate::shared::interfaces::ApiError;
use crate::utils::auth::{AuthPayload, UserJwt};
use crate::utils::responders::HbpResult;
use crate::utils::{
    constants::{cookies::*, headers::*},
    status_from,
};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_str_from_query_params(req: &Request) -> Option<String> {
    match req.query_value::<&str>("jwt") {
        Some(jwt) => jwt.map(|val| val.to_owned()).ok(),
        None => req
            .headers()
            .get_one(AUTHORIZATION)
            .map(|jwt_str| jwt_str.trim()["Bearer ".len()..].to_owned()),
    }
}

fn get_user_jwt(req: &Request) -> Option<UserJwt> {
    match jwt_str_from_query_params(req).or_else(|| {
        req.cookies()
            .get_private(USER_JWT)
            .map(|val| val.value().to_owned())
    }) {
        Some(token) => UserJwt::decode(&token).ok(),
        None => None,
    }
}

// ! FIXME: Dude, this is NOT correct...!
#[cfg(not(test))]
fn get_cookie(req: &Request, cookie_name: &str) -> Option<String> {
    req.cookies()
        .get_private(cookie_name)
        .map(|val| val.value().to_owned())
}
#[cfg(test)]
fn get_cookie(req: &Request, cookie_name: &str) -> Option<String> {
    req.cookies()
        .get(cookie_name)
        .map(|val| val.value().to_owned())
}

fn get_jwt(req: &Request) -> HbpResult<AuthPayload> {
    let jwt_str = jwt_str_from_query_params(req)
        .or_else(|| get_cookie(req, USER_JWT))
        .or_else(|| get_cookie(req, RESOURCE_JWT));

    match jwt_str {
        Some(token) => AuthPayload::decode(&token),
        None => Err(ApiError::unauthorized()),
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthPayload {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_jwt(req) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(e) => Outcome::Failure((status_from(e.status_code.clone()), e)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserJwt {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_user_jwt(req) {
            Some(jwt) => Outcome::Success(jwt),
            None => Outcome::Failure((Status::Unauthorized, ApiError::unauthorized())),
        }
    }
}
