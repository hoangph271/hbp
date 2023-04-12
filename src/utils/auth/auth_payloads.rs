use std::path::Path;

use httpstatus::StatusCode;
use jsonwebtoken::{decode, errors::Error, DecodingKey, TokenData, Validation};
use log::error;
use rocket::serde::{Deserialize, Serialize};

use crate::{
    data::models::users_model::DbUser,
    shared::interfaces::{ApiError, ApiResult},
    utils::{
        env::{self, from_env, jwt_secret},
        responders::{HbpError, HbpResult},
        timestamp_now,
    },
};

pub mod jwt {
    use crate::{
        shared::interfaces::ApiError,
        utils::{env::jwt_secret, responders::HbpResult},
    };
    use httpstatus::StatusCode;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;

    pub fn sign_jwt<T: Serialize>(payload: &T) -> HbpResult<String> {
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&jwt_secret()),
        )
        .map_err(|e| {
            ApiError::from_message(
                &format!("sign_jwt failed: {e}"),
                StatusCode::InternalServerError,
            )
            .into()
        })
    }
}

fn jwt_expires_in_ms() -> i64 {
    const MS_PER_HOUR: i64 = 60 * 60 * 1000;
    let jwt_expires_in_hours: i64 = env::from_env(env::EnvKey::JwtExpiresInHours)
        .parse()
        .unwrap_or_else(|e| panic!("JWT_EXPIRES_IN_HOURS must be an integer: {e}"));

    jwt_expires_in_hours * MS_PER_HOUR
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserJwt {
    pub exp: i64,
    pub sub: String,
    pub roles: Vec<String>,
}
impl UserJwt {
    pub fn sign_jwt(&self) -> Result<String, HbpError> {
        jwt::sign_jwt(&self)
    }

    pub fn set_sub(&mut self, sub: String) -> &UserJwt {
        self.sub = sub;
        self
    }

    pub fn decode(token: &str) -> Result<UserJwt, Error> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        decode::<UserJwt>(token, key, validation).map(|val| val.claims)
    }
}
impl Default for UserJwt {
    fn default() -> Self {
        Self {
            sub: Default::default(),
            roles: Default::default(),
            exp: timestamp_now() + jwt_expires_in_ms(),
        }
    }
}
impl From<DbUser> for UserJwt {
    fn from(db_user: DbUser) -> Self {
        UserJwt {
            sub: db_user.username,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourseJwt {
    pub exp: i64,
    pub sub: String,
    pub path: String,
}
impl Default for ResourseJwt {
    fn default() -> Self {
        Self {
            sub: Default::default(),
            path: Default::default(),
            exp: timestamp_now() + jwt_expires_in_ms(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthPayload {
    User(UserJwt),
    UserResource(ResourseJwt),
}

impl AuthPayload {
    pub fn username(&self) -> &str {
        match self {
            AuthPayload::User(jwt) => &jwt.sub,
            AuthPayload::UserResource(jwt) => &jwt.sub,
        }
    }

    pub fn is_root(&self) -> bool {
        if let Self::User(jwt) = self {
            jwt.sub.eq(from_env(env::EnvKey::RootUser))
        } else {
            false
        }
    }

    pub fn decode(token: &str) -> ApiResult<AuthPayload> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        let auth_payload: Result<AuthPayload, ApiError> = UserJwt::decode(token)
            .map(AuthPayload::User)
            .or_else(|_| decode::<ResourseJwt>(token, key, validation).map(|val| val.into()))
            .map_err(|_| {
                ApiError::from_message(
                    &format!("verify_jwt failed for {token}"),
                    StatusCode::Unauthorized,
                )
            });

        auth_payload
    }

    pub fn assert_username(&self, username: &str) -> ApiResult<()> {
        if username.eq(self.username()) {
            Ok(())
        } else {
            Err(ApiError::forbidden())
        }
    }

    pub fn match_path<F>(&self, path: &Path, user_assert: F) -> HbpResult<()>
    where
        F: FnOnce(&UserJwt, &Path) -> bool,
    {
        match self {
            AuthPayload::User(payload) => {
                if user_assert(payload, path) {
                    Ok(())
                } else {
                    Err(ApiError::forbidden().into())
                }
            }
            AuthPayload::UserResource(payload) => {
                let can_access = glob::Pattern::new(&payload.path)
                    .map_err(|e| {
                        error!("AuthPayload::match_path failed: {e}");
                        ApiError::forbidden()
                    })?
                    .matches(&path.to_string_lossy());

                if can_access {
                    Ok(())
                } else {
                    Err(ApiError::forbidden().into())
                }
            }
        }
    }

    pub fn sign(&self) -> Result<String, ApiError> {
        // * Maybe permission this, for now only root can sign url
        if !self.is_root() {
            return Err(ApiError::forbidden());
        };

        match self {
            AuthPayload::User(user_payload) => jwt::sign_jwt(user_payload),
            AuthPayload::UserResource(resource_payload) => jwt::sign_jwt(resource_payload),
        }
        .map_err(|e| e.api_error)
    }
}

impl From<TokenData<UserJwt>> for AuthPayload {
    fn from(token_data: TokenData<UserJwt>) -> Self {
        AuthPayload::User(token_data.claims)
    }
}
impl From<TokenData<ResourseJwt>> for AuthPayload {
    fn from(token_data: TokenData<ResourseJwt>) -> Self {
        AuthPayload::UserResource(token_data.claims)
    }
}
#[cfg(test)]
mod auth_payload_tests {
    use crate::utils::env::{from_env, EnvKey};

    use super::*;

    #[test]
    fn sign_jwt_as_root() {
        let root = from_env(EnvKey::RootUser);

        let auth_payload = AuthPayload::User(UserJwt {
            exp: 0,
            sub: root.to_owned(),
            roles: vec![],
        });

        let jwt = auth_payload.sign().unwrap();

        assert!(!jwt.is_empty())
    }

    #[test]
    fn sign_jwt_as_not_root() {
        let sub = "not-a-root".to_owned();

        let auth_payload = AuthPayload::User(UserJwt {
            exp: 0,
            sub,
            roles: vec![],
        });

        assert_eq!(auth_payload.sign(), Err(ApiError::forbidden()))
    }
}
