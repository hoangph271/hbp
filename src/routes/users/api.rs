use crate::data::user_orm::UserOrm;
use crate::shared::interfaces::{ApiErrorResponse, ApiItemResponse};
use crate::utils::responders::HbpResponse;
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode::BadRequest;
use log::*;
use rocket::post;
use rocket::serde::json::{Error as JsonError, Json};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

use super::shared::{attemp_signin, LoginBody};

#[derive(Deserialize, JsonSchema)]
pub struct SignupApiBody {
    username: String,
    password: String,
}
impl SignupApiBody {
    fn validate(&self) -> HbpResult<()> {
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
        } else {
            Ok(())
        }
    }
}

#[openapi]
#[post("/signup", data = "<signup_body>")]
pub async fn api_post_signup(
    signup_body: Result<Json<SignupApiBody>, JsonError<'_>>,
) -> HbpResponse {
    use crate::data::models::users_model::NewUser;

    match signup_body {
        Err(e) => {
            let error = match e {
                JsonError::Io(_) => "Can not read JSON".to_owned(),
                JsonError::Parse(_, e) => e.to_string(),
            };

            let errors = vec![error];
            ApiErrorResponse::bad_request(errors).into()
        }
        Ok(signup_body) => {
            if let Err(e) = signup_body.validate() {
                let errors = vec![e.msg];
                return ApiErrorResponse::bad_request(errors).into();
            }

            let new_user = NewUser {
                title: None,
                avatar_url: None,
                username: signup_body.username.clone(),
                hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
                    .expect("Hashing password failed"),
            };

            match UserOrm::from_env().create_user(new_user).await {
                Ok(new_user) => ApiItemResponse::ok(new_user).into(),
                Err(e) => {
                    let e: ApiErrorResponse = e.into();
                    e.into()
                }
            }
        }
    }
}

#[openapi]
#[post("/signin", data = "<signin_body>")]
pub async fn api_post_signin(signin_body: Json<LoginBody>) -> HbpResponse {
    let user_result = attemp_signin(&signin_body.username, &signin_body.password).await;

    match user_result {
        Ok(maybe_user) => match maybe_user {
            Some(user) => ApiItemResponse::ok(user).into(),
            None => ApiErrorResponse::unauthorized().into(),
        },
        Err(e) => {
            error!("{:?}", e);
            ApiErrorResponse::internal_server_error().into()
        }
    }
}
