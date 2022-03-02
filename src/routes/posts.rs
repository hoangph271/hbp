use crate::data::sqlite::DbConn;
use crate::data::{lib, models::posts_model};
use crate::utils::responders::HbpResponse;
use httpstatus::StatusCode;
use rocket::serde::json::Json;

#[get("/")]
pub async fn index(conn: DbConn) -> HbpResponse {
    conn.run(|conn| {
        let posts = lib::post_orm::get_posts(conn);

        HbpResponse::json(posts, None)
    })
    .await
}
#[get("/<post_id>")]
pub async fn find_one(post_id: String, conn: DbConn) -> HbpResponse {
    use lib::{post_orm, OrmError};

    conn.run(move |conn| match post_orm::get_one(conn, &post_id) {
        Ok(post) => HbpResponse::json(post, None),
        Err(e) => match e {
            OrmError::NotFound => HbpResponse::status(StatusCode::NotFound),
            OrmError::DieselError(e) => {
                error!("get_one failed: {:?}", e);
                HbpResponse::internal_server_error()
            }
        },
    })
    .await
}

#[delete("/<post_id>")]
pub async fn delete_one(post_id: String, conn: DbConn) -> HbpResponse {
    conn.run(move |conn| {
        lib::post_orm::delete_one(conn, &post_id);
        // TODO: Skip on 404, handle errors
        HbpResponse::status(StatusCode::Ok)
    })
    .await
}

#[post("/", data = "<new_post>")]
pub async fn create(new_post: Json<posts_model::NewPost>, conn: DbConn) -> HbpResponse {
    conn.run(
        |conn| match lib::post_orm::create_post(conn, new_post.into_inner()) {
            Ok(post) => HbpResponse::json(post, None),
            Err(e) => {
                error!("create_post failed: {e}");
                HbpResponse::internal_server_error()
            }
        },
    )
    .await
}

#[put("/", data = "<updated_post>")]
pub async fn update(updated_post: Json<posts_model::UpdatedPost>, conn: DbConn) -> HbpResponse {
    use lib::{post_orm, OrmError};

    conn.run(
        |conn| match post_orm::update_one(conn, updated_post.into_inner()) {
            Ok(_) => HbpResponse::status(StatusCode::Ok),
            Err(e) => match e {
                OrmError::NotFound => HbpResponse::status(StatusCode::NotFound),
                OrmError::DieselError(e) => {
                    error!("update fail: {:?}", e);
                    HbpResponse::internal_server_error()
                }
            },
        },
    )
    .await
}
