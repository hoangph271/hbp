use crate::data::lib::user_orm;
use crate::utils::auth::{AuthPayload, UserPayload};
use crate::utils::guards::auth_payload::USER_JWT_COOKIE;
use crate::utils::responders::{HbpContent, HbpResponse};
use crate::utils::types::{HbpError, HbpResult};
use crate::utils::{template, timestamp_now};
use mustache::Data;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::Route;

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
pub async fn post_login(login_body: Form<LoginBody>, jar: &CookieJar<'_>) -> HbpResponse {
    if let Some(user) = user_orm::find_one(&login_body.username).await.unwrap() {
        let is_password_matches =
            bcrypt::verify(&login_body.password, &user.hashed_password).unwrap_or(false);

        if is_password_matches {
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
pub async fn post_signup(signup_body: Form<SignupBody>) -> HbpResponse {
    if let Err(e) = signup_body.validate() {
        error!("{}", e);
        return HbpResponse::redirect(uri!("/users", signup));
    }

    use crate::data::models::users_model::NewUser;
    let new_user = NewUser {
        title: None,
        username: signup_body.username.clone(),
        hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
            .expect("Hashing password failed"),
    };

    if user_orm::create_user(new_user).await.unwrap().is_some() {
        HbpResponse::redirect(uri!("/users", login))
    } else {
        HbpResponse::redirect(uri!("/users", signup))
    }
}

pub fn users_routes() -> Vec<Route> {
    routes![index, login, signup, post_login, post_signup]
}
