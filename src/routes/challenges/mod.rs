use crate::{
    data::challenge_orm::ChallengeOrm,
    utils::responders::{wrap_api_handler, HbpApiResult, HbpError, HbpJson},
};
use hbp_types::{ApiError, ApiItem, ApiList, Challenge};
use okapi::openapi3::OpenApi;
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

#[openapi]
#[get("/")]
pub async fn api_get_challenges() -> HbpApiResult<Challenge> {
    let challenges = wrap_api_handler(|| async {
        let profile_orm = ChallengeOrm::default();

        let challenges = profile_orm.find().await?;

        Ok(challenges)
    })
    .await?;

    Ok(HbpJson::List(ApiList::ok(challenges)))
}

#[openapi]
#[get("/<challenge_id>")]
pub async fn api_get_challenge_by_id(challenge_id: &str) -> HbpApiResult<Challenge> {
    let challenge = wrap_api_handler(|| async {
        let profile_orm = ChallengeOrm::default();

        let challenge = profile_orm.find_one(&challenge_id).await?;

        Ok(challenge)
    })
    .await?
    .ok_or_else(|| HbpError {
        api_error: ApiError::not_found(),
    })?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_get_challenges, api_get_challenge_by_id]
}
