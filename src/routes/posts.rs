use crate::data::lib;
use crate::utils::responders::HbpResponse;
use crate::data::sqlite::establish_connection;

#[get("/")]
pub fn index() -> HbpResponse {
    let conn = establish_connection();

    let posts = lib::post_orm::get_posts(&conn);

    HbpResponse::json(posts, None)
}

#[delete("/<post_id>")]
pub fn delete_one(post_id: i32) -> HbpResponse {
    let conn = establish_connection();

    lib::post_orm::delete_one(&conn, post_id);

    HbpResponse::status(httpstatus::StatusCode::Ok)
}
