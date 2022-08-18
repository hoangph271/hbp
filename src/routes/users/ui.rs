use crate::data::models::users_model::DbUser;
use crate::data::user_orm::UserOrm;
use crate::routes::users::shared::{LoginBody, SignupBody};
use crate::utils::auth::{AuthPayload, UserPayload};
use crate::utils::constants::cookies;
use crate::utils::env::{from_env, EnvKey};
use crate::utils::responders::HbpResponse;
use crate::utils::template;
use crate::utils::template::{IndexLayoutData, TemplateRenderer};
use crate::utils::types::HbpError;
use log::*;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::time::{Duration, OffsetDateTime};
use rocket::{get, post, uri};
use serde::Serialize;

use super::shared::attemp_signin;

#[get("/")]
pub fn index(jwt: AuthPayload) -> HbpResponse {
    #[derive(Serialize, Debug)]
    struct RenderData {
        username: String,
    }

    match TemplateRenderer::new("users/profile.html".into()).to_html_page(
        RenderData {
            username: jwt.username().to_owned(),
        },
        IndexLayoutData::default()
            .title(jwt.username())
            .username(jwt.username()),
    ) {
        Ok(html) => HbpResponse::html(&html, None),
        Err(e) => e.into(),
    }
}

#[get("/login")]
pub fn login() -> HbpResponse {
    let html = TemplateRenderer::new("users/login.html".into())
        .to_html_page((), template::IndexLayoutData::default().title("Login"))
        .expect("render users/login.html failed");

    HbpResponse::html(&html, None)
}

#[get("/signup")]
pub fn signup() -> HbpResponse {
    match TemplateRenderer::new("users/signup.html".into())
        .to_html_page((), template::IndexLayoutData::default().title("Signup"))
    {
        Ok(html) => HbpResponse::html(&html, None),
        Err(e) => e.into(),
    }
}

#[post("/login", data = "<login_body>")]
pub async fn post_login(login_body: Form<LoginBody>, jar: &CookieJar<'_>) -> HbpResponse {
    match attemp_signin(&login_body.username, &login_body.password).await {
        Err(e) => {
            error!("attemp_signin() failed: {e}");
            HbpError::internal_server_error().into()
        }
        Ok(user) => match user {
            Some(user) => {
                let jwt = UserPayload::default().set_sub(user.username).sign_jwt();

                match jwt {
                    Ok(jwt) => {
                        let jwt_expires_in: i64 = from_env(EnvKey::JwtExpiresInHours)
                            .parse()
                            .unwrap_or_else(|e| panic!("parse JwtExpiresInHours failed: {e}"));

                        let expries_in =
                            OffsetDateTime::now_utc() + Duration::hours(jwt_expires_in);

                        let mut cookie = Cookie::new(cookies::USER_JWT, jwt);
                        cookie.set_expires(expries_in);

                        jar.add_private(cookie);

                        HbpResponse::redirect(uri!("/users", index))
                    }
                    Err(e) => e.into(),
                }
            }
            None => HbpResponse::redirect(uri!("/users", login)),
        },
    }
}

#[post("/signup", data = "<signup_body>")]
pub async fn post_signup(signup_body: Form<SignupBody>) -> HbpResponse {
    if let Err(e) = signup_body.validate() {
        error!("{}", e);
        return HbpResponse::redirect(uri!("/users", signup));
    }

    let new_user = DbUser {
        title: signup_body.username.clone(),
        username: signup_body.username.clone(),
        hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
            .expect("Hashing password failed"),
    };

    if UserOrm::default().create_user(new_user).await.is_ok() {
        HbpResponse::redirect(uri!("/users", login))
    } else {
        HbpResponse::redirect(uri!("/users", signup))
    }
}
