use crate::data::models::users_model::PutUser;
use crate::data::user_orm::UserOrm;
use crate::shared::interfaces::{ApiErrorResponse, ApiItemResponse};
use crate::utils::auth::UserPayload;
use crate::utils::responders::{wrap_api_handler, HbpResponse};
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode::BadRequest;
use rocket::serde::json::{Error as JsonError, Json};
use rocket::{post, put};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

use super::shared::{attemp_signin, LoginBody};

#[derive(Deserialize, JsonSchema)]
pub struct SignupApiPayload {
    username: String,
    password: String,
}
impl SignupApiPayload {
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
#[post("/signup", data = "<signup_payload>")]
pub async fn api_post_signup(
    signup_payload: Result<Json<SignupApiPayload>, JsonError<'_>>,
) -> HbpResponse {
    use crate::data::models::users_model::DbUser;

    let maybe_user = wrap_api_handler(|| async {
        let signup_body = signup_payload.map_err(|e| {
            let error = match e {
                JsonError::Io(_) => "Can not read JSON".to_owned(),
                JsonError::Parse(_, e) => e.to_string(),
            };

            HbpError::bad_request(error)
        })?;

        signup_body.validate()?;

        let new_user = UserOrm::default()
            .create_user(DbUser {
                title: signup_body.username.clone(),
                username: signup_body.username.clone(),
                hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
                    .expect("Hashing password failed"),
            })
            .await?;

        Ok(new_user)
    })
    .await;

    match maybe_user {
        Ok(user) => ApiItemResponse::ok(user).into(),
        Err(e) => ApiErrorResponse::from_hbp_error(e).into(),
    }
}

#[openapi]
#[put("/<username>", data = "<user>")]
pub async fn api_put_user(username: String, user: Json<PutUser>, jwt: UserPayload) -> HbpResponse {
    if username.ne(&jwt.sub) {
        return ApiErrorResponse::forbidden().into();
    }

    let user = user.into_inner();

    match UserOrm::default().update_user(user.clone()).await {
        Ok(_) => ApiItemResponse::ok(user).into(),
        Err(e) => ApiErrorResponse {
            status_code: e.status_code,
            errors: vec![e.message],
        }
        .into(),
    }
}

#[openapi]
#[post("/signin", data = "<signin_body>")]
pub async fn api_post_signin(signin_body: Json<LoginBody>) -> HbpResponse {
    let maybe_jwt: HbpResult<String> = wrap_api_handler(|| async {
        let user = attemp_signin(&signin_body.username, &signin_body.password)
            .await?
            .ok_or_else(HbpError::unauthorized)?;

        let user_jwt: UserPayload = user.into();

        user_jwt.sign_jwt()
    })
    .await;

    match maybe_jwt {
        Ok(jwt) => ApiItemResponse::ok(jwt).into(),
        Err(e) => ApiErrorResponse::from_hbp_error(e).into(),
    }
}
