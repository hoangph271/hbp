use crate::data::lib;
use crate::utils::responders::HbpResponse;

#[get("/")]
pub fn index() -> HbpResponse {
    use crate::data::sqlite::establish_connection;
    let connection = establish_connection();

    let posts = lib::post_orm::get_posts(&connection);

    HbpResponse::json(posts, None)
}
