use async_recursion::async_recursion;
use async_std::fs::{read_dir, ReadDir};
use async_std::path::PathBuf;
use async_std::prelude::*;
use httpstatus::StatusCode;
use log::error;
use mime_guess::Mime;
use okapi::openapi3::OpenApi;
use rand::{seq::SliceRandom, thread_rng};
use rocket::{get, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use std::path::Path;
use std::str::FromStr;

use crate::{
    shared::interfaces::ApiErrorResponse,
    utils::{
        auth::AuthPayload,
        env::{is_root, public_files_root},
        responders::HbpResponse,
        types::{HbpError, HbpResult},
    },
};

fn attemp_access(path: &Path, jwt: &Option<AuthPayload>) -> HbpResult<()> {
    fn is_private(path: &Path) -> bool {
        let is_in_public_folder = path.starts_with(public_files_root());
        if is_in_public_folder {
            return false;
        }

        true
    }

    if !is_private(path) {
        return Ok(());
    }

    match jwt {
        Some(jwt) => {
            jwt.match_path(
                path,
                |_, _| is_root(jwt.username()), // TODO: User jwt
            )
        }
        None => Err(HbpError::not_found()),
    }
}

#[openapi]
#[get("/random/raw?<mime>")]
pub async fn api_get_random_raw_file(
    mime: Option<String>,
    jwt: Option<AuthPayload>,
) -> HbpResult<HbpResponse> {
    let root = public_files_root();
    let mime = if let Some(mime) = mime {
        let mime = Mime::from_str(&mime).map_err(|e| {
            error!("{e:?}");
            HbpError::bad_request("file_type is malformed".to_owned())
        })?;

        Some(mime)
    } else {
        None
    };

    fn hbp_error_mapper(e: std::io::Error) -> HbpError {
        error!("{e:?}");
        HbpError::from_io_error(e, StatusCode::InternalServerError)
    }

    #[async_recursion]
    async fn get_matched_files(
        path: &Path,
        jwt: &Option<AuthPayload>,
        mime: &Option<Mime>,
    ) -> HbpResult<Vec<PathBuf>> {
        let mut matched_files = vec![];

        let mut entries: ReadDir = read_dir(path).await.map_err(hbp_error_mapper)?;

        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(hbp_error_mapper)?;
            let meta_data = entry.metadata().await.map_err(hbp_error_mapper)?;

            let can_access = attemp_access(entry.path().as_path().into(), jwt)
                .map(|()| true)
                .unwrap_or(false);

            if !can_access {
                continue;
            }

            if meta_data.is_file() {
                let match_file_type = mime_guess::from_path(entry.path()).iter().any(|item| {
                    let (item_type, item_sub_type) =
                        (item.type_().as_str(), item.subtype().as_str());

                    if let Some(file_mime) = mime {
                        let (sup_type, sub_type) =
                            (file_mime.type_().as_str(), file_mime.subtype().as_str());

                        if !sup_type.is_empty() && sup_type.ne("*") && item_type != sup_type {
                            return false;
                        }

                        if !sub_type.is_empty() && sub_type.ne("*") && item_sub_type != sub_type {
                            return false;
                        }

                        true
                    } else {
                        true
                    }
                });

                if match_file_type {
                    matched_files.push(entry.path());
                }
            } else {
                matched_files
                    .extend(get_matched_files(entry.path().as_path().into(), jwt, mime).await?);
            }
        }

        Ok(matched_files)
    }

    let mut files = get_matched_files(&root, &jwt, &mime).await?;

    files.shuffle(&mut thread_rng());

    match files.first() {
        Some(file) => Ok(HbpResponse::file(file.as_path().to_owned().into())),
        None => Err(HbpError::not_found()),
    }
}

#[openapi]
#[get("/raw?<path>")]
pub async fn api_get_raw_file(jwt: Option<AuthPayload>, path: Option<String>) -> HbpResponse {
    if let Some(path) = path {
        let path = Path::new(&path);

        if path.is_dir() {
            return ApiErrorResponse {
                status_code: StatusCode::UnprocessableEntity,
                errors: vec![format!("requested file at {path:?} is NOT a file")],
            }
            .into();
        }

        if !path.exists() {
            return ApiErrorResponse::not_found().into();
        }

        return match attemp_access(path, &jwt) {
            Ok(_) => HbpResponse::file(path.to_path_buf()),
            Err(e) => e.into(),
        };
    }

    ApiErrorResponse::not_found().into()
}

pub fn get_routes_and_docs(openapi_settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![openapi_settings: api_get_raw_file, api_get_random_raw_file]
}
