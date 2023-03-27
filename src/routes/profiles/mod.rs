use httpstatus::StatusCode;
use rocket::{get, routes, Route, State};
use sled::Db;

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

#[get("/")]
async fn api_get_profile(jwt: UserJwt, db: &State<Db>) -> HbpApiResult<DbProfile> {
    let profile = wrap_api_handler(|| async {
        let profile_orm = ProfileOrm::default();

        let mut maybe_profile = profile_orm.find_one(db, &jwt.sub).await?;

        if maybe_profile.is_none() {
            let user = UserOrm::default()
                .find_one(db, &jwt.sub)
                .await?
                .ok_or_else(|| DbError {
                    status_code: StatusCode::NotFound,
                    message: StatusCode::NotFound.reason_phrase().to_string(),
                })?;

            maybe_profile = Some(profile_orm.create_profile(db, user.into()).await?);
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

pub fn profiles_api_routes() -> Vec<Route> {
    routes![api_get_profile]
}
