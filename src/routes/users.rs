use crate::data::lib::user_orm;
use crate::data::models::users_model::User;
use crate::shared::interfaces::{ApiErrorResponse, ApiItemResponse};
use crate::utils::auth::{AuthPayload, UserPayload};
use crate::utils::guards::auth_payload::USER_JWT_COOKIE;
use crate::utils::responders::{HbpContent, HbpResponse};
use crate::utils::types::{HbpError, HbpResult};
use crate::utils::{template, timestamp_now};
use httpstatus::StatusCode::BadRequest;
use log::*;
use mustache::Data;
use okapi::openapi3::OpenApi;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::{Error as JsonError, Json};
use rocket::{get, post, routes, uri, FromForm, Route};
use rocket_okapi::settings::OpenApiSettings;
use rocket_okapi::{openapi, openapi_get_routes_spec};
use schemars::JsonSchema;
use serde::Deserialize;

#[get("/")]
fn index(jwt: AuthPayload) -> HbpResponse {
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
fn login() -> HbpResponse {
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
fn signup() -> HbpResponse {
    HbpResponse::ok(Some(HbpContent::Html(
        template::render_default_layout(
            "users/signup.html",
            Some(template::DefaultLayoutData::only_title("Signup")),
            None,
        )
        .expect("render users/signup.html failed"),
    )))
}

#[derive(FromForm, Deserialize, JsonSchema)]
struct LoginBody {
    username: String,
    password: String,
}
#[post("/login", data = "<login_body>")]
async fn post_login(login_body: Form<LoginBody>, jar: &CookieJar<'_>) -> HbpResponse {
    match attemp_signin(&login_body.username, &login_body.password).await {
        Some(user) => {
            let jwt = UserPayload::sign_jwt(&UserPayload {
                exp: timestamp_now(),
                sub: user.username,
                role: vec![],
            });

            jar.add_private(Cookie::new(USER_JWT_COOKIE, jwt));

            HbpResponse::redirect(uri!("/users", index))
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

#[openapi]
#[post("/signup", data = "<signup_body>")]
async fn api_post_signup(signup_body: Result<Json<SignupBody>, JsonError<'_>>) -> HbpResponse {
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
    match attemp_signin(&signin_body.username, &signin_body.password).await {
        Some(user) => ApiItemResponse::ok(user).into(),
        None => ApiErrorResponse::unauthorized().into(),
    }
}

pub fn users_routes() -> Vec<Route> {
    routes![index, login, signup, post_login, post_signup]
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_post_signup, api_post_signin]
}

async fn attemp_signin(username: &str, password: &str) -> Option<User> {
    if let Some(user) = user_orm::find_one(username).await.unwrap() {
        let is_password_matches = bcrypt::verify(password, &user.hashed_password).unwrap_or(false);

        if is_password_matches {
            Some(user)
        } else {
            None
        }
    } else {
        None
    }
}
