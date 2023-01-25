use crate::shared::interfaces::ApiError;
use crate::utils::auth::{AuthPayload, UserJwt};
use crate::utils::responders::{HbpError, HbpResult};
use crate::utils::{
    constants::{cookies::*, headers::AUTHORIZATION},
    status_from,
};
use rocket::http::{Cookie, HeaderMap, Status};
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_str_from_query_params(req: &Request) -> Option<String> {
    req.query_value::<&str>("jwt")
        .and_then(|val| val.ok())
        .map(|str| str.to_owned())
}
fn jwt_str_from_headers(headers: &HeaderMap) -> Option<String> {
    headers.get(AUTHORIZATION).next().and_then(|header| {
        let parts = header.split(' ').collect::<Vec<_>>();
        let jwt_str = parts.get(1);

        jwt_str.map(|str| str.to_string())
    })
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
    let token = jwt_str_from_query_params(req)
        .or_else(|| jwt_str_from_headers(req.headers()))
        .or_else(|| get_cookie(req, USER_JWT))
        .or_else(|| get_cookie(req, RESOURCE_JWT))
        .ok_or_else(|| HbpError::from(ApiError::unauthorized()))?;

    AuthPayload::decode(&token)
        .map_err(|e| e.into())
        .map(|jwt| {
            let cookies_name = match jwt {
                AuthPayload::User(_) => USER_JWT,
                AuthPayload::UserResource(_) => RESOURCE_JWT,
            };

            if get_cookie(req, cookies_name).is_none() {
                req.cookies().add_private(Cookie::new(cookies_name, token));
            }

            jwt
        })
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthPayload {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_jwt(req) {
            Ok(jwt) => Outcome::Success(jwt),
            Err(e) => Outcome::Failure((status_from(e.api_error.status_code.clone()), e.api_error)),
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
