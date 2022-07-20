use crate::utils::auth::{jwt, AuthPayload};
use crate::utils::constants;
use crate::utils::types::HbpError;
use httpstatus::StatusCode;
use rocket::http::{Cookie, Status};
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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_str = jwt_str_from_query_params(req);

        let jwt = if let Some(jwt_str) = jwt_str {
            let jwt = jwt::verify_jwt(&jwt_str).ok();

            if let Some(jwt) = jwt.clone() {
                match jwt {
                    AuthPayload::User(_) => {
                        req.cookies()
                            .add_private(Cookie::new(USER_JWT_COOKIE, jwt_str));
                    }
                    AuthPayload::UserResource(_) => {
                        req.cookies()
                            .add_private(Cookie::new(RESOURCE_JWT_COOKIE, jwt_str));
                    }
                }
            }

            jwt
        } else {
            let user_jwt_cookie = req.cookies().get_private(USER_JWT_COOKIE);
            let resource_jwt_cookie = req.cookies().get_private(RESOURCE_JWT_COOKIE);

            let jwt_from_cookies = if user_jwt_cookie.is_some() {
                user_jwt_cookie
            } else if resource_jwt_cookie.is_some() {
                resource_jwt_cookie
            } else {
                None
            };

            if let Some(jwt_str) = jwt_from_cookies {
                jwt::verify_jwt(jwt_str.value()).ok()
            } else {
                None
            }
        };

        if let Some(jwt) = jwt {
            Outcome::Success(jwt)
        } else {
            Outcome::Failure((
                Status::from_code(StatusCode::Unauthorized.as_u16()).unwrap(),
                HbpError::from_message("No valid jwt found", StatusCode::Unauthorized),
            ))
        }
    }
}
