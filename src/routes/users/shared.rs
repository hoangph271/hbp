use httpstatus::StatusCode::BadRequest;
use rocket::form::FromForm;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    data::{lib::DbResult, models::users_model::User, user_orm::user_orm},
    utils::types::{HbpError, HbpResult},
};

pub async fn attemp_signin(username: &str, password: &str) -> DbResult<Option<User>> {
    if let Some(user) = user_orm::find_one(username).await? {
        let is_password_matches = bcrypt::verify(password, &user.hashed_password).unwrap_or(false);

        if is_password_matches {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[derive(FromForm, Deserialize, JsonSchema)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

#[derive(FromForm, Deserialize, JsonSchema)]
pub struct SignupBody {
    pub username: String,
    pub password: String,
    #[field(name = "password-confirm")]
    pub password_confirm: String,
}
impl SignupBody {
    pub fn validate(&self) -> HbpResult<()> {
        if self.username.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "username can NOT be empty",
                BadRequest,
            ))
        } else if self.password.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "password can NOT be empty",
                BadRequest,
            ))
        } else if self.password.ne(&self.password_confirm) {
            HbpResult::Err(HbpError::from_message(
                "password & password_confirm does NOT mactch",
                BadRequest,
            ))
        } else {
            Ok(())
        }
    }
}
