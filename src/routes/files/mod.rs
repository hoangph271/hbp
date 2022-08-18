use httpstatus::StatusCode;
use okapi::openapi3::OpenApi;
use rocket::{get, FromFormField, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use schemars::JsonSchema;
use std::path::Path;

use crate::{
    shared::interfaces::ApiErrorResponse,
    utils::{
        auth::AuthPayload,
        env::{is_root, public_files_root},
        responders::HbpResponse,
    },
};

#[derive(Debug, PartialEq, FromFormField, JsonSchema)]
pub enum FileType {
    Image,
    PlainText,
    Markdown,
}

#[openapi]
#[get("/raw?<path>")]
pub async fn api_get_raw_file(
    // &<_random>&<_file_type>
    jwt: Option<AuthPayload>,
    path: Option<String>,
    // _random: Option<bool>,
    // _file_type: FileType,
) -> HbpResponse {
    if let Some(path) = path {
        let path = Path::new(&path);

        fn is_private(path: &Path) -> bool {
            let is_in_public_folder = path.starts_with(public_files_root());
            if is_in_public_folder {
                return false;
            }

            true
        }

        if !path.exists() {
            return ApiErrorResponse::not_found().into();
        }

        if path.is_dir() {
            return ApiErrorResponse {
                status_code: StatusCode::UnprocessableEntity,
                errors: vec![format!("requested file at {path:?} is NOT a file")],
            }
            .into();
        }

        if !is_private(&path) {
            return HbpResponse::file(path.to_path_buf());
        }

        return match jwt {
            Some(jwt) => {
                match jwt.match_path(
                    path, // TODO: User jwt
                    |_, _| is_root(jwt.username()),
                ) {
                    Ok(_) => HbpResponse::file(path.to_path_buf()),
                    Err(e) => ApiErrorResponse::from_status(e.status_code).into(),
                }
            }
            None => ApiErrorResponse::unauthorized().into(),
        };
    }

    todo!()
}

pub fn get_routes_and_docs(openapi_settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![openapi_settings: api_get_raw_file]
}
