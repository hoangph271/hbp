use httpstatus::StatusCode;
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::error;
use rocket::serde::{Deserialize, Serialize};

use crate::utils::{env, timestamp_now, types::HbpError};

pub mod jwt {
    use crate::utils::types::{HbpError, HbpResult};
    use httpstatus::StatusCode;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;

    use super::jwt_secret;

    pub fn sign_jwt<T: Serialize>(payload: &T) -> HbpResult<String> {
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&jwt_secret()),
        )
        .map_err(|e| {
            HbpError::from_message(
                &format!("sign_jwt failed: {e}"),
                StatusCode::InternalServerError,
            )
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserPayload {
    pub exp: i64,
    #[serde(rename = "expiresIn")]
    pub expires_in: String,
    pub sub: String,
    pub roles: Vec<String>,
}
impl UserPayload {
    pub fn sign_jwt(&self) -> Result<String, HbpError> {
        jwt::sign_jwt(&self)
    }

    pub fn set_sub(&mut self, sub: String) -> &UserPayload {
        self.sub = sub;
        self
    }
}
impl Default for UserPayload {
    fn default() -> Self {
        Self {
            expires_in: format!(
                "{}h",
                env::from_env(env::EnvKey::JwtExpiresInHours).to_owned()
            ),
            sub: Default::default(),
            roles: Default::default(),
            exp: timestamp_now(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResoucePayload {
    pub exp: i64,
    #[serde(rename = "expiresIn")]
    pub expires_in: String,
    pub sub: String,
    pub path: String,
}
impl Default for UserResoucePayload {
    fn default() -> Self {
        Self {
            expires_in: format!(
                "{}h",
                env::from_env(env::EnvKey::JwtExpiresInHours).to_owned()
            ),
            sub: Default::default(),
            path: Default::default(),
            exp: timestamp_now(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthPayload {
    User(UserPayload),
    UserResource(UserResoucePayload),
}

fn jwt_secret() -> Vec<u8> {
    use crate::utils::env::{from_env, EnvKey};
    let key = from_env(EnvKey::JwtSecret);

    key.as_bytes().into()
}

impl AuthPayload {
    pub fn username(&self) -> &str {
        match self {
            AuthPayload::User(jwt) => &jwt.sub,
            AuthPayload::UserResource(jwt) => &jwt.sub,
        }
    }

    pub fn decode(token: &str) -> Result<AuthPayload, HbpError> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        match decode::<UserPayload>(token, key, validation) {
            Ok(result) => {
                return Ok(AuthPayload::User(result.claims));
            }
            Err(e) => {
                error!("decode::<UserPayload> fail: {e}");
            }
        }

        match decode::<UserResoucePayload>(token, key, validation) {
            Ok(result) => {
                return Ok(AuthPayload::UserResource(result.claims));
            }
            Err(e) => {
                error!("decode::<UserResoucePayload> fail: {}", e);
            }
        }

        Err(HbpError::from_message(
            &format!("verify_jwt failed for {token}"),
            StatusCode::Unauthorized,
        ))
    }

    pub fn match_path<F>(&self, path: &str, user_assert: Option<F>) -> bool
    where
        F: FnOnce(&UserPayload, &str) -> bool,
    {
        match self {
            AuthPayload::User(payload) => {
                if let Some(user_assert) = user_assert {
                    user_assert(payload, path)
                } else {
                    false
                }
            }
            AuthPayload::UserResource(payload) => {
                if payload.path.is_empty() {
                    // ? Yeah, this one to make previously used JWT works
                    // FIXME: Maybe remove this, or use '*'
                    return true;
                }

                match glob::Pattern::new(&payload.path) {
                    Ok(pattern) => pattern.matches(path),
                    Err(_) => false,
                }
            }
        }
    }
}
