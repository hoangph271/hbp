use crate::data::{lib::user_orm, sqlite::DbConn};
use crate::utils::jwt::{sign_jwt, JwtPayload};
use crate::utils::responders::{HbpContent, HbpResponse};
use crate::utils::template;
use httpstatus::StatusCode;
use rocket::form::Form;

#[get("/")]
pub fn index(jwt: JwtPayload) -> HbpResponse {
    return HbpResponse::text(&*format!("hello {:?}", jwt), StatusCode::Ok);
}

#[get("/login")]
pub fn login() -> HbpResponse {
    let html = template::render_from_template("users/login.html", &None)
        .expect("render users/login.html failed");
    HbpResponse::ok(Some(HbpContent::Html(html)))
}

#[derive(FromForm)]
pub struct LoginBody {
    username: String,
    password: String,
}
#[post("/login", data = "<login_body>")]
pub async fn post_login(login_body: Form<LoginBody>, conn: DbConn) -> HbpResponse {
    conn.run(move |conn| {
        if let Ok(user) = user_orm::find_one_by_username(conn, &login_body.username) {
            let is_password_matches =
                bcrypt::verify(&login_body.password, &user.hashed_password).unwrap_or(false);

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
    })
    .await
}
