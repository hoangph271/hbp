use crate::{
    shared::interfaces::{ApiItem, ApiResult},
    utils::auth::AuthPayload,
};

use rocket::get;
use rocket_okapi::openapi;
use std::path::PathBuf;

#[openapi]
#[get("/users/<username>/<sub_path..>")]
pub(super) async fn api_user_markdown(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
) -> ApiResult<ApiItem<String>> {
    jwt.assert_username(username)?;

    todo!()
}
