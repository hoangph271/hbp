use crate::data::sqlite::establish_connection;
use crate::data::{lib, models};
use crate::utils::responders::HbpResponse;
use httpstatus::StatusCode;
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

    HbpResponse::status(StatusCode::Ok)
}

#[post("/", data = "<new_post>")]
pub fn create(new_post: Json<models::NewPost>) -> HbpResponse {
    let conn = establish_connection();

    match lib::post_orm::create_post(&conn, new_post.into_inner()) {
        Ok(post) => {
            HbpResponse::json(post, None)
        },
        Err(e) => {
            error!("{e}");
            HbpResponse::internal_server_error()
        }
    }
}

#[put("/", data = "<updated_post>")]
pub fn update(updated_post: Json<models::UpdatedPost>) -> HbpResponse {
    use lib::post_orm;
    use lib::post_orm::OrmError;
    let conn = establish_connection();

    match post_orm::update_one(&conn, updated_post.into_inner()) {
        Ok(_) => HbpResponse::status(StatusCode::Ok),
        Err(e) => match e {
            OrmError::NotFound => HbpResponse::status(StatusCode::NotFound),
            OrmError::DieselError(e) => {
                error!("update fail: {:?}", e);
                HbpResponse::internal_server_error()
            }
        },
    }
}
