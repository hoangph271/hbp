use httpstatus::StatusCode::BadRequest;
use rocket::{form::FromForm, State};
use schemars::JsonSchema;
use serde::Deserialize;
use sled::Db;

use crate::{
    data::{lib::DbResult, models::users_model::DbUser, user_orm::UserOrm},
    shared::interfaces::ApiError,
    utils::responders::HbpResult,
};

pub async fn attemp_signin(username: &str, password: &str, db: &State<Db>) -> DbResult<Option<DbUser>> {
    if let Some(user) = UserOrm::default().find_one(username, db).await? {
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
    #[field(validate = len(2..10))]
    pub username: String,
    #[field(validate = len(2..10))]
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
            Err(ApiError::from_message("username can NOT be empty", BadRequest).into())
        } else if self.password.is_empty() {
            Err(ApiError::from_message("password can NOT be empty", BadRequest).into())
        } else if self.password.ne(&self.password_confirm) {
            Err(
                ApiError::from_message("password & password_confirm does NOT mactch", BadRequest)
                    .into(),
            )
        } else {
            Ok(())
        }
    }
}
