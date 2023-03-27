use rocket::{routes, Route};

mod api;
mod shared;
mod ui;

use api::*;
use ui::*;

pub fn users_routes() -> Vec<Route> {
    routes![index, login, signup, post_login, post_signup]
}

pub fn users_api_routes() -> Vec<Route> {
    routes![api_post_signup, api_post_signin, api_put_user]
}
