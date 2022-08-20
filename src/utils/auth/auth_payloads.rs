use std::path::Path;

use httpstatus::StatusCode;
use jsonwebtoken::{decode, errors::Error, DecodingKey, TokenData, Validation};
use log::error;
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

use crate::{
    data::models::users_model::DbUser,
    shared::interfaces::ApiError,
    utils::{env, timestamp_now, types::HbpResult},
};

pub mod jwt {
    use crate::{shared::interfaces::ApiError, utils::types::HbpResult};
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
            ApiError::from_message(
                &format!("sign_jwt failed: {e}"),
                StatusCode::InternalServerError,
            )
        })
    }
}

fn jwt_expires_in_ms() -> i64 {
    const MS_PER_HOUR: i64 = 60 * 60 * 1000;
    let jwt_expires_in_hours: i64 = env::from_env(env::EnvKey::JwtExpiresInHours)
        .parse()
        .expect("JWT_EXPRIES_IN_HOURS must be an integer");

    jwt_expires_in_hours * MS_PER_HOUR
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserPayload {
    pub exp: i64,
    pub sub: String,
    pub roles: Vec<String>,
}
impl UserPayload {
    pub fn sign_jwt(&self) -> Result<String, ApiError> {
        jwt::sign_jwt(&self)
    }

    pub fn set_sub(&mut self, sub: String) -> &UserPayload {
        self.sub = sub;
        self
    }

    pub fn decode(token: &str) -> Result<UserPayload, Error> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        decode::<UserPayload>(token, key, validation).map(|val| val.claims)
    }
}
impl Default for UserPayload {
    fn default() -> Self {
        Self {
            sub: Default::default(),
            roles: Default::default(),
            exp: timestamp_now() + jwt_expires_in_ms(),
        }
    }
}
impl<'r> OpenApiFromRequest<'r> for UserPayload {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}
impl From<DbUser> for UserPayload {
    fn from(db_user: DbUser) -> Self {
        UserPayload {
            sub: db_user.username,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResoucePayload {
    pub exp: i64,
    pub sub: String,
    pub path: String,
}
impl Default for UserResoucePayload {
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

    pub fn decode(token: &str) -> Result<AuthPayload, ApiError> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        let auth_payload: Result<AuthPayload, ApiError> = UserPayload::decode(token)
            .map(AuthPayload::User)
            .or_else(|_| decode::<UserResoucePayload>(token, key, validation).map(|val| val.into()))
            .map_err(|_| {
                ApiError::from_message(
                    &format!("verify_jwt failed for {token}"),
                    StatusCode::Unauthorized,
                )
            });

        auth_payload
    }

    pub fn match_path<F>(&self, path: &Path, user_assert: F) -> HbpResult<()>
    where
        F: FnOnce(&UserPayload, &Path) -> bool,
    {
        match self {
            AuthPayload::User(payload) => {
                if user_assert(payload, path) {
                    Ok(())
                } else {
                    Err(ApiError::forbidden())
                }
            }
            AuthPayload::UserResource(payload) => {
                let can_access = glob::Pattern::new(&payload.path)
                    .map_err(|e| {
                        error!("{e}");
                        ApiError::forbidden()
                    })?
                    .matches(&path.to_string_lossy());

                if can_access {
                    Ok(())
                } else {
                    Err(ApiError::forbidden())
                }
            }
        }
    }

    pub fn sign(&self) -> Result<String, ApiError> {
        match self {
            AuthPayload::User(user_payload) => jwt::sign_jwt(user_payload),
            AuthPayload::UserResource(resource_payload) => jwt::sign_jwt(resource_payload),
        }
    }
}

impl From<TokenData<UserPayload>> for AuthPayload {
    fn from(token_data: TokenData<UserPayload>) -> Self {
        AuthPayload::User(token_data.claims)
    }
}
impl From<TokenData<UserResoucePayload>> for AuthPayload {
    fn from(token_data: TokenData<UserResoucePayload>) -> Self {
        AuthPayload::UserResource(token_data.claims)
    }
}
impl<'r> OpenApiFromRequest<'r> for AuthPayload {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}
