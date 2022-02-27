use crate::utils::types::HbpError;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use rocket::serde::Deserialize;
use sha2::Sha256;

use crate::utils::{constants, status_from};
use httpstatus::StatusCode;
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_secret() -> Hmac<Sha256> {
    use crate::utils::env::{from_env, EnvKey};
    let key = from_env(EnvKey::JwtSecret);

    Hmac::new_from_slice(key.as_bytes()).expect("Invalid JWT_KEY")
}

pub fn verify_jwt(token_str: &str) -> Result<JwtPayload, HbpError> {
    token_str.verify_with_key(&jwt_secret()).map_err(|e| {
        error!("{e}");

        HbpError::from_message(&*format!("verify_with_key() failed with {token_str}"))
    })
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct JwtPayload {
    iat: usize,
    sub: String,
    name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(jwt_header) = req.headers().get_one(constants::headers::AUTHORIZATION) {
            let jwt_str = &jwt_header.trim()["Bearer ".len()..];

            return match verify_jwt(jwt_str) {
                Ok(claims) => Outcome::Success(claims),
                Err(e) => {
                    let error = HbpError::from_message(&*format!("Invalid JWT: {:?}", e));
                    Outcome::Failure((status_from(StatusCode::Unauthorized), error))
                }
            };
        }

        let error = HbpError::from_message(&format!(
            "Header `{}` not found",
            constants::headers::AUTHORIZATION
        ));
        Outcome::Failure((status_from(StatusCode::Unauthorized), error))
    }
}
