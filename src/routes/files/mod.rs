use async_recursion::async_recursion;
use async_std::fs::{read_dir, ReadDir};
use async_std::path::PathBuf as AsyncPathBuf;
use async_std::prelude::*;
use httpstatus::StatusCode;
use log::error;
use mime_guess::Mime;
use okapi::openapi3::OpenApi;
use rand::{seq::SliceRandom, thread_rng};
use rocket::{get, uri, Route};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::shared::interfaces::{ApiError, ApiResult};
use crate::utils::create_thumbnail;
use crate::utils::{
    auth::AuthPayload,
    env::{files_root, is_root, public_files_root},
    responders::HbpResponse,
};

fn attempt_access(path: &Path, jwt: &Option<AuthPayload>) -> ApiResult<()> {
    fn is_private(path: &Path) -> bool {
        let is_in_public_folder = path.starts_with(public_files_root());

        if is_in_public_folder {
            return false;
        }

        true
    }

    if !path.starts_with(files_root()) {
        return Err(if jwt.is_some() {
            ApiError::forbidden()
        } else {
            ApiError::unauthorized()
        });
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
        None => Err(ApiError::forbidden()),
    }
}

fn assert_raw_file(path: &Path) -> ApiResult<&Path> {
    if path.is_dir() {
        Err(ApiError {
            status_code: StatusCode::UnprocessableEntity,
            errors: vec!["requested file at {path:?} is NOT a file".to_owned()],
        })
    } else if !path.exists() {
        println!("{path:?}");
        Err(ApiError::not_found())
    } else {
        Ok(path)
    }
}

#[openapi]
#[get("/random/raw?<mime>&<preview>")]
pub async fn api_get_random_file(
    mime: Option<String>,
    jwt: Option<AuthPayload>,
    preview: Option<bool>,
) -> ApiResult<HbpResponse> {
    let mime = if let Some(mime) = mime {
        let mime = Mime::from_str(&mime).map_err(|e| {
            error!("{e:?}");
            ApiError::bad_request(vec!["file_type is malformed".to_owned()])
        })?;

        Some(mime)
    } else {
        None
    };

    #[async_recursion]
    async fn get_matched_files(
        path: &Path,
        jwt: &Option<AuthPayload>,
        mime: &Option<Mime>,
    ) -> ApiResult<Vec<AsyncPathBuf>> {
        let mut matched_files = vec![];

        let mut entries: ReadDir = read_dir(path).await?;

        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let meta_data = entry.metadata().await?;

            let can_access = attempt_access(entry.path().as_path().into(), jwt)
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

    let root = public_files_root();
    let mut files = get_matched_files(&root, &jwt, &mime).await?;

    files.shuffle(&mut thread_rng());

    match files.first() {
        Some(file_path) => {
            let file_path = PathBuf::from(file_path);

            let uri = if preview.unwrap_or(false) {
                uri!(
                    "/api/v1/files",
                    api_get_preview_file(path = file_path, from_root = Some(true))
                )
            } else {
                uri!(
                    "/api/v1/files",
                    api_get_raw_file(path = file_path, from_root = Some(true))
                )
            };

            Ok(HbpResponse::redirect(uri))
        }
        None => Err(ApiError::not_found()),
    }
}

#[openapi]
#[get("/preview/<path..>?<from_root>")]
pub async fn api_get_preview_file(
    jwt: Option<AuthPayload>,
    path: PathBuf,
    from_root: Option<bool>,
) -> ApiResult<HbpResponse> {
    let path = if from_root.unwrap_or(false) {
        PathBuf::from("/").join(path)
    } else {
        path
    };

    attempt_access(&path, &jwt)?;
    assert_raw_file(&path)?;

    Ok(HbpResponse::temp_file(create_thumbnail(&path)?))
}

#[openapi]
#[get("/raw/<path..>?<from_root>", rank = 2)]
pub async fn api_get_raw_file(
    jwt: Option<AuthPayload>,
    path: PathBuf,
    from_root: Option<bool>,
) -> ApiResult<HbpResponse> {
    let path = if from_root.unwrap_or(false) {
        PathBuf::from("/").join(path)
    } else {
        path
    };

    let path = files_root().join(path);

    attempt_access(&path, &jwt)?;
    assert_raw_file(&path)?;

    Ok(HbpResponse::file(path))
}

pub fn get_routes_and_docs(openapi_settings: &OpenApiSettings) -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        openapi_settings: api_get_raw_file,
        api_get_preview_file,
        api_get_random_file
    ]
}
