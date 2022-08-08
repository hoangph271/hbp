use crate::data::lib::{user_orm, DbResult};
use crate::data::models::users_model::User;
use crate::shared::interfaces::{ApiErrorResponse, ApiItemResponse};
use crate::utils::auth::{AuthPayload, UserPayload};
use crate::utils::constants::cookies;
use crate::utils::env::{from_env, EnvKey};
use crate::utils::responders::HbpResponse;
use crate::utils::template;
use crate::utils::template::{IndexLayoutData, TemplateRenderer};
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode::BadRequest;
use log::*;
use okapi::openapi3::OpenApi;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::{Error as JsonError, Json};
use rocket::time::{Duration, OffsetDateTime};
use rocket::{get, post, routes, uri, FromForm, Route};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[get("/")]
fn index(jwt: AuthPayload) -> HbpResponse {
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
fn login() -> HbpResponse {
    let html = TemplateRenderer::new("users/login.html".into())
        .to_html_page((), template::IndexLayoutData::default().title("Login"))
        .expect("render users/login.html failed");

    HbpResponse::html(&html, None)
}
#[get("/signup")]
fn signup() -> HbpResponse {
    match TemplateRenderer::new("users/signup.html".into())
        .to_html_page((), template::IndexLayoutData::default().title("Signup"))
    {
        Ok(html) => HbpResponse::html(&html, None),
        Err(e) => e.into(),
    }
}

#[derive(FromForm, Deserialize, JsonSchema)]
struct LoginBody {
    username: String,
    password: String,
}
#[post("/login", data = "<login_body>")]
async fn post_login(login_body: Form<LoginBody>, jar: &CookieJar<'_>) -> HbpResponse {
    match attemp_signin(&login_body.username, &login_body.password)
        .await
        .unwrap()
    {
        Some(user) => {
            let jwt = UserPayload::default().set_sub(user.username).sign_jwt();

            match jwt {
                Ok(jwt) => {
                    let jwt_expires_in: i64 = from_env(EnvKey::JwtExpiresInHours).parse().unwrap();
                    let expries_in = OffsetDateTime::now_utc() + Duration::hours(jwt_expires_in);

                    let mut cookie = Cookie::new(cookies::USER_JWT, jwt);
                    cookie.set_expires(expries_in);

                    jar.add_private(cookie);

                    HbpResponse::redirect(uri!("/users", index))
                }
                Err(e) => e.into(),
            }
        }
        None => HbpResponse::redirect(uri!("/users", login)),
    }
}

#[derive(FromForm, Deserialize, JsonSchema)]
struct SignupBody {
    username: String,
    password: String,
    #[field(name = "password-confirm")]
    password_confirm: String,
}
impl SignupBody {
    fn validate(&self) -> HbpResult<()> {
        if self.username.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "username can NOT be empty",
                BadRequest,
            ))
        } else if self.password.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "password can NOT be empty",
                BadRequest,
            ))
        } else if self.password.ne(&self.password_confirm) {
            HbpResult::Err(HbpError::from_message(
                "password & password_confirm does NOT mactch",
                BadRequest,
            ))
        } else {
            Ok(())
        }
    }
}

#[post("/signup", data = "<signup_body>")]
async fn post_signup(signup_body: Form<SignupBody>) -> HbpResponse {
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

    if user_orm::create_user(new_user).await.is_ok() {
        HbpResponse::redirect(uri!("/users", login))
    } else {
        HbpResponse::redirect(uri!("/users", signup))
    }
}

// * APIs

#[derive(Deserialize, JsonSchema)]
struct SignupApiBody {
    username: String,
    password: String,
}
impl SignupApiBody {
    fn validate(&self) -> HbpResult<()> {
        if self.username.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "username can NOT be empty",
                BadRequest,
            ))
        } else if self.password.is_empty() {
            HbpResult::Err(HbpError::from_message(
                "password can NOT be empty",
                BadRequest,
            ))
        } else {
            Ok(())
        }
    }
}

#[openapi]
#[post("/signup", data = "<signup_body>")]
async fn api_post_signup(signup_body: Result<Json<SignupApiBody>, JsonError<'_>>) -> HbpResponse {
    use crate::data::models::users_model::NewUser;

    match signup_body {
        Err(e) => {
            let error = match e {
                JsonError::Io(_) => "Can not read JSON".to_owned(),
                JsonError::Parse(_, e) => e.to_string(),
            };

            let errors = vec![error];
            ApiErrorResponse::bad_request(errors).into()
        }
        Ok(signup_body) => {
            if let Err(e) = signup_body.validate() {
                let errors = vec![e.msg];
                return ApiErrorResponse::bad_request(errors).into();
            }

            let new_user = NewUser {
                title: None,
                username: signup_body.username.clone(),
                hashed_password: bcrypt::hash(&signup_body.password, bcrypt::DEFAULT_COST)
                    .expect("Hashing password failed"),
            };

            match user_orm::create_user(new_user).await {
                Ok(new_user) => ApiItemResponse::ok(new_user).into(),
                Err(e) => {
                    let e: ApiErrorResponse = e.into();
                    e.into()
                }
            }
        }
    }
}

#[openapi]
#[post("/signin", data = "<signin_body>")]
async fn api_post_signin(signin_body: Json<LoginBody>) -> HbpResponse {
    let user_result = attemp_signin(&signin_body.username, &signin_body.password).await;

    match user_result {
        Ok(maybe_user) => match maybe_user {
            Some(user) => ApiItemResponse::ok(user).into(),
            None => ApiErrorResponse::unauthorized().into(),
        },
        Err(e) => {
            error!("{:?}", e);
            ApiErrorResponse::internal_server_error().into()
        }
    }
}

pub fn users_routes() -> Vec<Route> {
    routes![index, login, signup, post_login, post_signup]
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_post_signup, api_post_signin]
}

async fn attemp_signin(username: &str, password: &str) -> DbResult<Option<User>> {
    if let Some(user) = user_orm::find_one(username).await? {
        let is_password_matches = bcrypt::verify(password, &user.hashed_password).unwrap_or(false);

        if is_password_matches {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
