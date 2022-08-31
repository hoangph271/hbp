use okapi::openapi3::OpenApi;
use rocket::{routes, Route};
use rocket_okapi::{openapi_get_routes_spec, settings::OpenApiSettings};
use std::path::{Path, PathBuf};

mod api;
mod ui;

pub use api::*;
pub use ui::*;

use crate::utils::auth::UserJwt;

fn assert_payload_access(payload: &UserJwt, path: &Path) -> bool {
    let prefix = PathBuf::from("markdown")
        .join("users")
        .join(payload.sub.clone())
        .to_string_lossy()
        .into_owned();

    path.starts_with(&*prefix)
}

fn markdown_path_from(username: &str, sub_path: &Path) -> (String, PathBuf) {
    let file_path = PathBuf::from("markdown")
        .join("users")
        .join(username)
        .join(sub_path);

    (file_path.to_string_lossy().to_string(), file_path)
}

pub fn markdown_routes() -> Vec<Route> {
    routes![
        markdown_file,
        user_markdown_file,
        user_markdown_editor,
        user_default
    ]
}

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![settings: api_user_markdown]
}
