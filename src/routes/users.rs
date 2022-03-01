use crate::data::{lib::user_orm, sqlite::establish_connection};
use crate::utils::jwt::{sign_jwt, JwtPayload};
use crate::utils::responders::{HbpContent, HbpResponse};
use httpstatus::StatusCode;
use rocket::serde::json::Json;
use serde::Deserialize;

#[get("/")]
pub fn index(jwt: JwtPayload) -> HbpResponse {
    return HbpResponse::text(&*format!("hello {:?}", jwt), StatusCode::Ok);
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
        let is_password_matches =
            bcrypt::verify(login_body.password, &user.hashed_password).unwrap_or(false);

        if is_password_matches {
            #[derive(serde::Serialize)]
            struct JwtRes {
                jwt: String,
            }

            let exp = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::minutes(60))
                .unwrap()
                .timestamp();

            let jwt = sign_jwt(JwtPayload {
                sub: user.username,
                role: Vec::new(),
                exp,
            });

            return HbpResponse::json(JwtRes { jwt }, None);
        }

        return HbpResponse::status(StatusCode::Unauthorized);
    }

    HbpResponse::status(StatusCode::Unauthorized)
}