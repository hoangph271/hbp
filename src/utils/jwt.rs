use crate::utils::types::HbpError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::serde::{Deserialize, Serialize};

use crate::utils::constants;
use httpstatus::StatusCode;
use rocket::http::{Cookie, Status};
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_secret() -> String {
    use crate::utils::env::{from_env, EnvKey};
    let key = from_env(EnvKey::JwtSecret);

    key.to_owned()
}

pub fn verify_jwt(token_str: &str) -> Result<JwtPayload, HbpError> {
    let key = &DecodingKey::from_secret(jwt_secret().as_bytes());
    let validation = &Validation::default();

    match decode::<UserPayload>(token_str, key, validation) {
        Ok(result) => {
            return Ok(JwtPayload::User(result.claims));
        }
        Err(e) => {
            error!("{e}");
        }
    }

    match decode::<UserResoucePayload>(token_str, key, validation) {
        Ok(result) => {
            return Ok(JwtPayload::UserResouce(result.claims));
        }
        Err(e) => {
            error!("{e}");
        }
    }

    Err(HbpError::from_message(&format!(
        "verify_jwt failed for {token_str}"
    )))
}
pub fn sign_jwt(payload: JwtPayload) -> String {
    encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .unwrap()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserPayload {
    pub exp: i64,
    pub sub: String,
    pub role: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct UserResoucePayload {
    pub exp: i64,
    pub sub: String,
}
pub const RESOURCE_JWT_COOKIE: &str = "resource-jwt";

#[derive(Debug, Deserialize, Serialize)]
pub enum JwtPayload {
    User(UserPayload),
    UserResouce(UserResoucePayload),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let mut jwt_str = req
            .headers()
            .get_one(constants::headers::AUTHORIZATION)
            .map(|jwt_str| jwt_str.trim()["Bearer ".len()..].to_owned());

        let resource_jwt_cookies = req.cookies().get_private(RESOURCE_JWT_COOKIE);

        if jwt_str.is_none() {
            if let Some(_jwt_str) = req.query_value::<&str>("jwt") {
                jwt_str = _jwt_str.map(|val| val.to_owned()).ok();

                if resource_jwt_cookies.is_none() && jwt_str.is_some() {
                    req.cookies()
                        .add_private(Cookie::new(RESOURCE_JWT_COOKIE, jwt_str.clone().unwrap()))
                }
            } else if let Some(val) = resource_jwt_cookies {
                jwt_str = Some(val.value().to_owned())
            }
        }

        if let Some(jwt_str) = jwt_str {
            return match verify_jwt(&jwt_str) {
                Ok(claims) => Outcome::Success(claims),
                Err(e) => {
                    let error = HbpError::from_message(&*format!("Invalid JWT: {:?}", e));
                    Outcome::Failure((
                        Status::from_code(StatusCode::Unauthorized.as_u16()).unwrap(),
                        error,
                    ))
                }
            };
        }

        let error = HbpError::from_message(&format!(
            "Header `{}` not found",
            constants::headers::AUTHORIZATION
        ));
        Outcome::Failure((
            Status::from_code(StatusCode::Unauthorized.as_u16()).unwrap(),
            error,
        ))
    }
}

impl JwtPayload {
    pub fn sub_from(jwt_payload: JwtPayload) -> String {
        match jwt_payload {
            JwtPayload::User(jwt) => jwt.sub,
            JwtPayload::UserResouce(jwt) => jwt.sub,
        }
    }
}
