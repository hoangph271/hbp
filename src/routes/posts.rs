use crate::data::lib::post_orm::*;
use crate::data::models::posts_model::Post;
use crate::data::{lib, models::posts_model};
use crate::utils::responders::{HbpApiResult, HbpJson, HbpResponse};
use hbp_types::{ApiItem, ApiList};
use httpstatus::StatusCode;
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, Route};

#[get("/")]
pub async fn index() -> HbpApiResult<Post> {
    let posts = lib::post_orm::get_posts().await?;

    Ok(HbpJson::List(ApiList::ok(posts)))
}
#[get("/<post_id>")]
pub async fn find_one(post_id: String) -> HbpApiResult<Post> {
    let post = get_one(&post_id)?;

    todo!()
}

#[delete("/<post_id>")]
pub async fn api_delete_one(post_id: String) -> HbpApiResult<()> {
    delete_one(&post_id);

    Ok(HbpJson::Empty)
}

#[post("/", data = "<new_post>")]
pub async fn create(new_post: Json<posts_model::NewPost>) -> HbpApiResult<Post> {
    let post = create_post(new_post.into_inner()).expect("create_post() failed");

    Ok(ApiItem::ok(post).into())
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
    routes![index, find_one, api_delete_one, create, update]
}
