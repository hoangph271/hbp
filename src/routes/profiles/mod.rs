use httpstatus::StatusCode;
use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

use crate::{
    data::{
        lib::DbError, models::profiles_model::DbProfile, profile_orm::ProfileOrm, user_orm::UserOrm,
    },
    shared::interfaces::ApiItem,
    utils::{
        auth::UserJwt,
        responders::{wrap_api_handler, HbpApiResult, HbpJson},
    },
};

#[openapi]
#[get("/")]
pub async fn api_get_profile(jwt: UserJwt) -> HbpApiResult<DbProfile> {
    let profile = wrap_api_handler(|| async {
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
    .await?;

    let item = ApiItem::ok(profile);
    Ok(HbpJson::Item(item))
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_profile]
}
