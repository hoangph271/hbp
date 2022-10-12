use crate::{
    data::challenge_orm::ChallengeOrm,
    utils::responders::{wrap_api_handler, HbpApiResult, HbpError, HbpJson},
};
use hbp_types::{ApiError, ApiItem, ApiList, Challenge};
use okapi::openapi3::OpenApi;
use rocket::{get, post, put, serde::json::Json, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

#[openapi]
#[get("/")]
pub async fn api_get_challenges() -> HbpApiResult<Challenge> {
    let challenges = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        let challenges = orm.find().await?;

        Ok(challenges)
    })
    .await?;

    Ok(HbpJson::List(ApiList::ok(challenges)))
}

#[openapi]
#[post("/", data = "<new_challenge>")]
pub async fn api_post_challenge(new_challenge: Json<Challenge>) -> HbpApiResult<Challenge> {
    let new_challenge = new_challenge.into_inner();
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        orm.create(new_challenge).await.map_err(|e| e.into())
    })
    .await?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

#[openapi]
#[put("/", data = "<challenge>")]
pub async fn api_put_challenge(challenge: Json<Challenge>) -> HbpApiResult<Challenge> {
    let challenge = challenge.into_inner();
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        orm.update(challenge).await.map_err(|e| e.into())
    })
    .await?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

#[openapi]
#[get("/<challenge_id>")]
pub async fn api_get_challenge_by_id(challenge_id: &str) -> HbpApiResult<Challenge> {
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        let challenge = orm.find_one(challenge_id).await?;

        Ok(challenge)
    })
    .await?
    .ok_or_else(|| HbpError {
        api_error: ApiError::not_found(),
    })?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: api_get_challenges,
        api_get_challenge_by_id,
        api_post_challenge,
        api_put_challenge
    ]
}
