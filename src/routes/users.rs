use crate::data::{lib::user_orm, sqlite::DbConn};
use crate::utils::auth::{AuthPayload, UserPayload, USER_JWT_COOKIE};
use crate::utils::responders::{HbpContent, HbpResponse};
use crate::utils::types::{HbpError, HbpResult};
use crate::utils::{template, timestamp_now};
use mustache::Data;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};

#[get("/")]
pub fn index(jwt: AuthPayload) -> HbpResponse {
    HbpResponse::html(
        &template::render_default_layout(
            "users/profile.html",
            Some(template::DefaultLayoutData::only_title(jwt.username()).username(jwt.username())),
            Some(template::simple_data_from(vec![(
                "username".to_owned(),
                Data::String(jwt.username().to_owned()),
            )])),
        )
        .unwrap(),
        None,
    )
}

#[get("/login")]
pub fn login() -> HbpResponse {
    HbpResponse::html(
        &template::render_default_layout(
            "users/login.html",
            Some(template::DefaultLayoutData::only_title("Login")),
            None,
        )
        .expect("render users/login.html failed"),
        None,
    )
}
#[get("/signup")]
pub fn signup() -> HbpResponse {
    HbpResponse::ok(Some(HbpContent::Html(
        template::render_default_layout(
            "users/signup.html",
            Some(template::DefaultLayoutData::only_title("Signup")),
            None,
        )
        .expect("render users/signup.html failed"),
    )))
}

#[derive(FromForm)]
pub struct LoginBody {
    username: String,
    password: String,
}
#[post("/login", data = "<login_body>")]
pub async fn post_login(
    login_body: Form<LoginBody>,
    conn: DbConn,
    jar: &CookieJar<'_>,
) -> HbpResponse {
    let res = conn
        .run(move |conn| {
            if let Ok(user) = user_orm::find_one_by_username(conn, &login_body.username) {
                let is_password_matches =
                    bcrypt::verify(&login_body.password, &user.hashed_password).unwrap_or(false);

                if is_password_matches {
                    Some(user)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .await;

    if let Some(user) = res {
        let jwt = UserPayload::sign_jwt(&UserPayload {
            exp: timestamp_now(),
            sub: user.username,
            role: vec![],
        });

        jar.add_private(Cookie::new(USER_JWT_COOKIE, jwt));

        HbpResponse::redirect(uri!("/users", index))
    } else {
        HbpResponse::redirect(uri!("/users", login))
    }
}

#[derive(FromForm)]
pub struct SignupBody {
    username: String,
    password: String,
    #[field(name = "password-confirm")]
    password_confirm: String,
}
impl SignupBody {
    fn validate(&self) -> HbpResult<()> {
        if self.username.is_empty() {
            HbpResult::Err(HbpError::from_message("username can NOT be empty"))
        } else if self.password.is_empty() {
            HbpResult::Err(HbpError::from_message("password can NOT be empty"))
        } else if self.password.ne(&self.password_confirm) {
            HbpResult::Err(HbpError::from_message(
                "password & password_confirm does NOT mactch",
            ))
        } else {
            Ok(())
        }
    }
}

#[post("/signup", data = "<signup_body>")]
pub async fn post_signup(signup_body: Form<SignupBody>, conn: DbConn) -> HbpResponse {
    if let Err(e) = signup_body.validate() {
        error!("{}", e);
        return HbpResponse::redirect(uri!("/users", signup));
    }

    conn.run(move |conn| {
        use crate::data::models::users_model::NewUser;
        let new_user = NewUser {
            title: None,
            username: &signup_body.username,
            hashed_password: &bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
                .expect("Hashing password failed"),
        };

        if user_orm::create_user(conn, new_user).is_ok() {
            HbpResponse::redirect(uri!("/users", login))
        } else {
            HbpResponse::redirect(uri!("/users", signup))
        }
    })
    .await
}
