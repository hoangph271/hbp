use crate::data::models::users_model::DbUser;
use crate::data::user_orm::UserOrm;
use crate::routes::users::shared::{LoginBody, SignupBody};
use crate::shared::interfaces::ApiError;
use crate::utils::auth::{AuthPayload, UserJwt};
use crate::utils::constants::cookies;
use crate::utils::env::{from_env, EnvKey};
use crate::utils::responders::{HbpResponse, HbpResult};
use crate::utils::template;
use crate::utils::template::{IndexLayout, Templater};
use httpstatus::StatusCode;
use log::*;
use rocket::form::Form;
use rocket::http::uri::{Origin, Uri};
use rocket::http::{Cookie, CookieJar};
use rocket::time::{Duration, OffsetDateTime};
use rocket::{get, post, uri};
use serde::Serialize;

use super::shared::attemp_signin;

#[get("/")]
pub fn index(jwt: AuthPayload) -> HbpResult<HbpResponse> {
    #[derive(Serialize, Debug)]
    struct RenderData {
        username: String,
    }

    let html = Templater::new("users/profile.html".into()).to_html_page(
        RenderData {
            username: jwt.username().to_owned(),
        },
        IndexLayout::default()
            .title(jwt.username().to_owned())
            .username(jwt.username()),
    )?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/login?<redirect_url>")]
pub fn login(redirect_url: Option<String>) -> HbpResult<HbpResponse> {
    #[derive(Serialize, Debug)]
    struct RenderData {
        redirect_url: String,
    }

    let html = Templater::new("users/login.html".into()).to_html_page(
        RenderData {
            redirect_url: redirect_url.unwrap_or_default(),
        },
        template::IndexLayout::from_title("Login".to_owned()),
    )?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/signup")]
pub fn signup() -> HbpResult<HbpResponse> {
    let html = Templater::new("users/signup.html".into())
        .to_html_page((), template::IndexLayout::from_title("Signup".to_owned()))?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[post("/login?<redirect_url>", data = "<login_body>")]
pub async fn post_login(
    login_body: Form<LoginBody>,
    jar: &CookieJar<'_>,
    redirect_url: Option<String>,
) -> HbpResponse {
    match attemp_signin(&login_body.username, &login_body.password).await {
        Err(e) => {
            error!("attemp_signin() failed: {e}");
            ApiError::internal_server_error().into()
        }
        Ok(user) => match user {
            Some(user) => {
                let jwt = UserJwt::default().set_sub(user.username).sign_jwt();

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

                        let redirect_url = redirect_url.unwrap_or_else(|| "/users/".to_owned());

                        let uri = Uri::parse::<Origin>(&redirect_url)
                            .map(|uri| {
                                uri.origin()
                                    .map(|uri| uri.to_owned())
                                    .unwrap_or_else(|| uri!("/users", index))
                            })
                            .unwrap_or_else(|e| {
                                error!("Uri::parse() `{redirect_url}` failed: {e}");
                                uri!("/users", index)
                            });

                        HbpResponse::redirect(uri)
                    }
                    Err(e) => HbpResponse::from_error_status(e.api_error.status_code),
                }
            }
            None => HbpResponse::redirect(uri!("/users", login(redirect_url = redirect_url))),
        },
    }
}

#[post("/signup", data = "<signup_body>")]
pub async fn post_signup(signup_body: Form<SignupBody>) -> HbpResponse {
    if let Err(e) = signup_body.validate() {
        error!("{e:?}");
        return HbpResponse::redirect(uri!("/users", signup));
    }

    let new_user = DbUser {
        title: signup_body.username.clone(),
        username: signup_body.username.clone(),
        hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
            .map_err(|e| ApiError::internal_server_error().append_error(e.to_string()))
            .unwrap_or_else(|e| panic!("bcrypt::hash failed: {e:?}")),
    };

    if UserOrm::default().create_user(new_user).await.is_ok() {
        HbpResponse::redirect(uri!("/users", login(_)))
    } else {
        HbpResponse::redirect(uri!("/users", signup))
    }
}
