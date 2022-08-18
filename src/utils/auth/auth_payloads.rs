use httpstatus::StatusCode;
use jsonwebtoken::{decode, errors::Error, DecodingKey, TokenData, Validation};
use rocket::serde::{Deserialize, Serialize};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

use crate::{
    data::models::users_model::DbUser,
    utils::{env, timestamp_now, types::HbpError},
};

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
    pub fn sign_jwt(&self) -> Result<String, HbpError> {
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

    pub fn decode(token: &str) -> Result<AuthPayload, HbpError> {
        let key = &DecodingKey::from_secret(&jwt_secret());
        let validation = &Validation::default();

        let auth_payload: Result<AuthPayload, HbpError> = UserPayload::decode(token)
            .map(AuthPayload::User)
            .or_else(|_| decode::<UserResoucePayload>(token, key, validation).map(|val| val.into()))
            .map_err(|_| {
                HbpError::from_message(
                    &format!("verify_jwt failed for {token}"),
                    StatusCode::Unauthorized,
                )
            });

        auth_payload
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
            AuthPayload::UserResource(payload) => match glob::Pattern::new(&payload.path) {
                Ok(pattern) => pattern.matches(path),
                Err(_) => false,
            },
        }
    }

    pub fn sign(&self) -> Result<String, HbpError> {
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
