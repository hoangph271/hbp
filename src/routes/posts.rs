use crate::data::sqlite::establish_connection;
use crate::data::{lib, models};
use crate::utils::responders::HbpResponse;
use rocket::serde::json::Json;

#[get("/")]
pub fn index() -> HbpResponse {
    let conn = establish_connection();

    let posts = lib::post_orm::get_posts(&conn);

    HbpResponse::json(posts, None)
}

#[delete("/<post_id>")]
pub fn delete_one(post_id: &str) -> HbpResponse {
    let conn = establish_connection();

    lib::post_orm::delete_one(&conn, post_id);

    HbpResponse::status(httpstatus::StatusCode::Ok)
}

#[post("/", data = "<new_post>")]
pub fn create(new_post: Json<models::NewPost>) -> HbpResponse {
    let conn = establish_connection();

    lib::post_orm::create_post(&conn, new_post.into_inner());

    HbpResponse::status(httpstatus::StatusCode::Created)
}
