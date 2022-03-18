use crate::utils::types::HbpError;
use rocket::serde::{Deserialize, Serialize};

use crate::utils::constants;
use httpstatus::StatusCode;
use rocket::http::{Cookie, Status};
use rocket::request::{FromRequest, Outcome, Request};
use serde_json::json;

pub const RESOURCE_JWT_COOKIE: &str = "resource-jwt";
pub const USER_JWT_COOKIE: &str = "user-jwt";

pub mod jwt {
    use crate::utils::auth::{AuthPayload, UserPayload, UserResoucePayload};
    use crate::utils::types::HbpError;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

    fn jwt_secret() -> String {
        use crate::utils::env::{from_env, EnvKey};
        let key = from_env(EnvKey::JwtSecret);

        key.to_owned()
    }

    pub fn verify_jwt(token_str: &str) -> Result<AuthPayload, HbpError> {
        let key = &DecodingKey::from_secret(jwt_secret().as_bytes());
        let validation = &Validation::default();

        match decode::<UserPayload>(token_str, key, validation) {
            Ok(result) => {
                return Ok(AuthPayload::User(result.claims));
            }
            Err(e) => {
                error!("{e}");
            }
        }

        match decode::<UserResoucePayload>(token_str, key, validation) {
            Ok(result) => {
                return Ok(AuthPayload::UserResource(result.claims));
            }
            Err(e) => {
                error!("{e}");
            }
        }

        Err(HbpError::from_message(&format!(
            "verify_jwt failed for {token_str}"
        )))
    }
    pub fn sign_jwt(payload: &str) -> String {
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .unwrap()
    }
}

fn timestamp_now() -> i64 {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(60))
        .unwrap()
        .timestamp()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserPayload {
    pub exp: i64,
    pub sub: String,
    pub role: Vec<String>,
}
impl UserPayload {
    pub fn sign_jwt(role: Vec<String>, sub: String) -> String {
        jwt::sign_jwt(&json!({
            "exp": timestamp_now(),
            "role": role,
            "sub": sub,
        }).to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResoucePayload {
    pub exp: i64,
    pub sub: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthPayload {
    User(UserPayload),
    UserResource(UserResoucePayload),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // TODO: Fix this shit...! :"/
        let jwt_from_query = req.query_value::<&str>("jwt");
        let jwt_from_header = req
            .headers()
            .get_one(constants::headers::AUTHORIZATION)
            .map(|jwt_str| jwt_str.trim()["Bearer ".len()..].to_owned());

        let jwt_str = if let Some(_jwt_str) = jwt_from_query {
            _jwt_str.map(|val| val.to_owned()).ok()
        } else if jwt_from_header.is_some() {
            jwt_from_header
        } else {
            None
        };

        let user_jwt_cookie = req.cookies().get_private(USER_JWT_COOKIE);
        let resource_jwt_cookie = req.cookies().get_private(RESOURCE_JWT_COOKIE);

        let jwt = if let Some(jwt_str) = jwt_str.clone() {
            let jwt = jwt::verify_jwt(&jwt_str).ok();

            if let Some(jwt) = jwt.clone() {
                match jwt {
                    AuthPayload::User(_) => {
                        if user_jwt_cookie.is_none() {
                            req.cookies()
                                .add_private(Cookie::new(USER_JWT_COOKIE, jwt_str.clone()));
                        }
                    }
                    AuthPayload::UserResource(_) => {
                        if resource_jwt_cookie.is_none() {
                            req.cookies()
                                .add_private(Cookie::new(RESOURCE_JWT_COOKIE, jwt_str.clone()));
                        }
                    }
                }
            }

            jwt
        } else {
            info!("{:?}", user_jwt_cookie);

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
                HbpError::from_message("No valid jwt found"),
            ))
        }
    }
}

impl AuthPayload {
    pub fn username(&self) -> &String {
        match self {
            AuthPayload::User(jwt) => &jwt.sub,
            AuthPayload::UserResource(jwt) => &jwt.sub,
        }
    }
}
