use crate::shared::{ApiError, ApiItem, ApiList, Challenge};
use crate::{
    data::challenge_orm::ChallengeOrm,
    utils::{
        auth::AuthPayload,
        responders::{wrap_api_handler, HbpApiResult, HbpError, HbpJson},
    },
};
use okapi::openapi3::OpenApi;
use rocket::{delete, get, post, put, serde::json::Json, Route, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use sled::Db;

#[openapi]
#[get("/")]
pub async fn api_get_challenges(db: &State<Db>) -> HbpApiResult<Challenge> {
    let challenges = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        let challenges = orm.find(db).await?;

        Ok(challenges)
    })
    .await?;

    Ok(HbpJson::List(ApiList::ok(challenges)))
}

#[openapi]
#[post("/", data = "<new_challenge>")]
pub async fn api_post_challenge(
    new_challenge: Json<Challenge>,
    _jwt: AuthPayload,
    db: &State<Db>,
) -> HbpApiResult<Challenge> {
    let new_challenge = new_challenge.into_inner();
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        orm.create(db, new_challenge).await.map_err(|e| e.into())
    })
    .await?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

#[openapi]
#[put("/", data = "<challenge>")]
pub async fn api_put_challenge(
    challenge: Json<Challenge>,
    _jwt: AuthPayload,
    db: &State<Db>,
) -> HbpApiResult<Challenge> {
    let challenge = challenge.into_inner();
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        orm.update(db, challenge).await.map_err(|e| e.into())
    })
    .await?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

#[openapi]
#[get("/<challenge_id>")]
pub async fn api_get_challenge_by_id(
    challenge_id: &str,
    db: &State<Db>,
) -> HbpApiResult<Challenge> {
    let challenge = wrap_api_handler(|| async {
        let orm = ChallengeOrm::default();

        let challenge = orm.find_one(db, challenge_id).await?;

        Ok(challenge)
    })
    .await?
    .ok_or_else(|| HbpError {
        api_error: ApiError::not_found(),
    })?;

    Ok(HbpJson::Item(ApiItem::ok(challenge)))
}

#[openapi]
#[delete("/<challenge_id>")]
pub async fn api_delete_challenge_by_id(
    challenge_id: &str,
    _jwt: AuthPayload,
    db: &State<Db>,
) -> HbpApiResult<()> {
    wrap_api_handler(|| async {
        ChallengeOrm::default().delete(db, challenge_id).await?;

        Ok(())
    })
    .await?;

    Ok(HbpJson::Item(ApiItem::ok(())))
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        settings: api_get_challenges,
        api_get_challenge_by_id,
        api_post_challenge,
        api_put_challenge,
        api_delete_challenge_by_id
    ]
}
