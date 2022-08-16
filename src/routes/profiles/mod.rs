use httpstatus::StatusCode;
use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

use crate::{
    data::{lib::DbError, profile_orm::ProfileOrm, user_orm::UserOrm},
    shared::interfaces::{ApiErrorResponse, ApiItemResponse},
    utils::{
        auth::UserPayload,
        responders::{wrap_api_handler, HbpResponse},
    },
};

#[openapi]
#[get("/")]
pub async fn api_get_profile(jwt: UserPayload) -> HbpResponse {
    let maybe_profile = wrap_api_handler(|| async {
        let profile_orm = ProfileOrm::default();

        let mut maybe_profile = profile_orm.find_one(&jwt.sub).await?;

        if maybe_profile.is_none() {
            let user = UserOrm::default()
                .find_one(&jwt.sub)
                .await?
                .ok_or_else(|| DbError {
                    status_code: StatusCode::NotFound,
                    message: StatusCode::NotFound.reason_phrase().to_string(),
                })?;

            maybe_profile = Some(profile_orm.create_profile(user.into()).await?);
        }

        maybe_profile.ok_or_else(|| {
            DbError::internal_server_error(format!(
                "find_one profile failed for username `{}`",
                &jwt.sub
            ))
            .into()
        })
    })
    .await;

    match maybe_profile {
        Ok(profile) => ApiItemResponse::ok(profile).into(),
        Err(e) => ApiErrorResponse::from_hbp_error(e).into(),
    }
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_profile]
}
