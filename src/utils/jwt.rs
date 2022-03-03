use crate::utils::types::HbpError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::serde::{Deserialize, Serialize};

use crate::utils::{constants, status_from};
use httpstatus::StatusCode;
use rocket::request::{FromRequest, Outcome, Request};

fn jwt_secret() -> String {
    use crate::utils::env::{from_env, EnvKey};
    let key = from_env(EnvKey::JwtSecret);

    key.to_owned()
}

pub fn verify_jwt(token_str: &str) -> Result<JwtPayload, HbpError> {
    match decode::<JwtPayload>(
        token_str,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    ) {
        Ok(jwt_payload) => Ok(jwt_payload.claims),
        Err(e) => {
            println!("{:?}", e);
            Err(HbpError::from_message("verify_jwt failed for ${token_str}"))
        }
    }
}
pub fn sign_jwt(payload: JwtPayload) -> String {
    encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .unwrap()
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct JwtPayload {
    pub exp: i64,
    pub sub: String,
    pub role: Vec<String>,
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
