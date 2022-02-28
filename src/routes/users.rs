use crate::data::{lib::user_orm, sqlite::establish_connection};
use crate::utils::jwt::JwtPayload;
use crate::utils::responders::{HbpContent, HbpResponse};
use httpstatus::StatusCode;
use rocket::serde::json::Json;
use serde::Deserialize;

#[get("/")]
pub fn index(jwt: JwtPayload) -> HbpResponse {
    return HbpResponse::text(&*format!("hello {:?}", jwt), httpstatus::StatusCode::Ok);
}

#[get("/login")]
pub fn login() -> HbpResponse {
    HbpResponse::ok(Some(HbpContent::Plain("login".to_owned())))
}

#[derive(Deserialize)]
pub struct LoginBody<'r> {
    username: &'r str,
    password: &'r str,
}
#[post("/login", data = "<login_body>")]
pub fn post_login(login_body: Json<LoginBody<'_>>) -> HbpResponse {
    let conn = establish_connection();

    if let Ok(user) = user_orm::find_one_by_username(&conn, login_body.username) {
        // TODO: Check for password...?
        return HbpResponse::ok(Some(HbpContent::Plain(format!(
            "Hello, {}...!",
            user.username
        ))));
    }

    HbpResponse::status(StatusCode::Unauthorized)
}
