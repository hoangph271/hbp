use crate::data::{lib, models::posts_model};
use crate::shared::interfaces::{ApiError, ApiResult};
use crate::utils::responders::HbpResponse;
use crate::utils::types::HbpResult;
use httpstatus::StatusCode;
use log::*;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route};

#[get("/")]
pub async fn index() -> HbpResult<HbpResponse> {
    let posts = lib::post_orm::get_posts().await;

    HbpResponse::json(posts, None)
}
#[get("/<post_id>")]
pub async fn find_one(post_id: String) -> HbpResult<HbpResponse> {
    use lib::{post_orm, OrmError};

    match post_orm::get_one(&post_id) {
        Ok(post) => HbpResponse::json(post, None),
        Err(e) => match e {
            OrmError::NotFound => Err(ApiError::from_status(StatusCode::NotFound)),
        },
    }
}

#[delete("/<post_id>")]
pub async fn delete_one(post_id: String) -> HbpResponse {
    lib::post_orm::delete_one(&post_id);
    // TODO: Skip on 404, handle errors
    HbpResponse::from_error_status(StatusCode::Ok)
}

#[post("/", data = "<new_post>")]
pub async fn create(new_post: Json<posts_model::NewPost>) -> ApiResult<HbpResponse> {
    match lib::post_orm::create_post(new_post.into_inner()) {
        Ok(post) => HbpResponse::json(post, None),
        Err(_) => {
            error!("create_post failed");
            Err(ApiError::from_status(StatusCode::InternalServerError))
        }
    }
}

#[put("/", data = "<updated_post>")]
pub async fn update(updated_post: Json<posts_model::UpdatedPost>) -> HbpResponse {
    use lib::{post_orm, OrmError};

    match post_orm::update_one(updated_post.into_inner()) {
        Ok(_) => HbpResponse::from_error_status(StatusCode::Ok), // FIME: error_status is NOT correct
        Err(e) => match e {
            OrmError::NotFound => HbpResponse::from_error_status(StatusCode::NotFound),
        },
    }
}

pub fn posts_routes() -> Vec<Route> {
    routes![index, find_one, delete_one, create, update]
}
