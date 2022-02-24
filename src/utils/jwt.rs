use crate::utils::types::HbpError;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use rocket::serde::Deserialize;
use sha2::Sha256;
use std::env;

use crate::utils::{constants, status_from};
use httpstatus::StatusCode;
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_key() -> Hmac<Sha256> {
    let key = match env::var("JWT_KEY") {
        Ok(key) => key,
        Err(e) => {
            error!("{e}");
            error!("`JWT_KEY` is NOT in the env, using default value");
            String::from("JWT_KEY")
        }
    };
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).expect("Invalid JWT_KEY");

    key
}

pub fn verify_jwt(token_str: &str) -> Option<JwtPayload> {
    let claims: Result<JwtPayload, HbpError> = token_str.verify_with_key(&jwt_key()).map_err(|e| {
        error!("{e}");

        HbpError::from_message("verify_with_key() failed with {token_str}")
    });

    println!("{:?}", claims);

    claims.ok()
}
#[derive(Debug, Deserialize)]
pub struct JwtPayload {
    // iat: usize,
    // sub: String,
    // name: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtPayload {
    type Error = HbpError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(jwt_header) = req.headers().get_one(constants::headers::AUTHORIZATION) {
            let jwt_str = &jwt_header.trim()["Bearer ".len()..];

            return match verify_jwt(jwt_str) {
                Some(claims) => Outcome::Success(claims),
                None => {
                    let error = HbpError::from_message("Invalid JWT");
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
