use rocket::serde::{Deserialize, Serialize};

pub mod jwt {
    use crate::utils::auth::{AuthPayload, UserPayload, UserResoucePayload};
    use crate::utils::types::HbpError;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::Serialize;

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
                error!("decode::<UserPayload> fail: {e}");
            }
        }

        match decode::<UserResoucePayload>(token_str, key, validation) {
            Ok(result) => {
                return Ok(AuthPayload::UserResource(result.claims));
            }
            Err(e) => {
                error!("decode::<UserResoucePayload> fail: {}", e);
            }
        }

        Err(HbpError::from_message(&format!(
            "verify_jwt failed for {token_str}"
        )))
    }
    pub fn sign_jwt<T: Serialize>(payload: T) -> String {
        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserPayload {
    pub exp: i64,
    pub sub: String,
    pub role: Vec<String>,
}
impl UserPayload {
    pub fn sign_jwt(&self) -> String {
        jwt::sign_jwt(&self)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResoucePayload {
    pub exp: i64,
    pub sub: String,
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthPayload {
    User(UserPayload),
    UserResource(UserResoucePayload),
}

impl AuthPayload {
    pub fn username(&self) -> &str {
        match self {
            AuthPayload::User(jwt) => &jwt.sub,
            AuthPayload::UserResource(jwt) => &jwt.sub,
        }
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
