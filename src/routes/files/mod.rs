use async_recursion::async_recursion;
use async_std::fs::read_dir;
use async_std::path::PathBuf as AsyncPathBuf;

use crate::shared::{ApiItem, Directory};
use futures::StreamExt;
use log::error;
use mime_guess::Mime;
use rand::{seq::SliceRandom, thread_rng};
use rocket::{get, routes, uri, Route};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::shared::interfaces::ApiError;
use crate::utils::create_thumbnail;
use crate::utils::responders::{HbpApiResult, HbpJson, HbpResult};
use crate::utils::{
    auth::AuthPayload,
    env::{files_root, is_root, public_files_root},
    responders::HbpResponse,
};

fn attempt_access(path: &Path, jwt: &Option<AuthPayload>) -> HbpResult<()> {
    fn is_private(path: &Path) -> bool {
        let is_in_public_folder = path.starts_with(public_files_root());

        if is_in_public_folder {
            return false;
        }

        true
    }

    if is_private(path) {
        match jwt {
            Some(jwt) => {
                jwt.match_path(
                    path,
                    // FIXME: Only root can access for now
                    |_, _| is_root(jwt.username()),
                )
            }
            None => Err(ApiError::forbidden().into()),
        }
    } else {
        Ok(())
    }
}

fn assert_file_access(path: &Path) -> HbpResult<&Path> {
    if path.is_dir() {
        Err(ApiError::unprocessable_entity()
            .append_error(format!("requested file at {path:?} is NOT a file"))
            .into())
    } else if !path.exists() {
        Err(ApiError::not_found().into())
    } else {
        Ok(path)
    }
}

fn assert_directory_access(path: &Path) -> HbpResult<&Path> {
    if path.is_file() {
        Err(ApiError::unprocessable_entity()
            .append_error(format!(
                "requested directory at {path:?} is NOT a directory"
            ))
            .into())
    } else if !path.exists() {
        Err(ApiError::not_found().into())
    } else {
        Ok(path)
    }
}

#[get("/dir/<path..>")]
async fn api_get_directory(path: PathBuf, jwt: Option<AuthPayload>) -> HbpApiResult<Directory> {
    let path = files_root().join(path);

    attempt_access(&path, &jwt)?;
    assert_directory_access(&path)?;

    let item = Directory {
        children: read_dir(path)
            .await?
            .map(|entry| {
                entry.unwrap().path().to_string_lossy()[files_root().to_string_lossy().len() + 1..]
                    .to_owned()
            })
            .collect()
            .await,
    };

    Ok(HbpJson::Item(ApiItem::ok(item)))
}

#[get("/random/raw?<mime>&<preview>")]
async fn api_get_random_file(
    mime: Option<String>,
    jwt: Option<AuthPayload>,
    preview: Option<bool>,
) -> HbpResult<HbpResponse> {
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
    ) -> HbpResult<Vec<AsyncPathBuf>> {
        let mut matched_files = vec![];

        match read_dir(path).await {
            Ok(mut entries) => {
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
                        let match_file_type =
                            mime_guess::from_path(entry.path()).iter().any(|item| {
                                let (item_type, item_sub_type) =
                                    (item.type_().as_str(), item.subtype().as_str());

                                if let Some(file_mime) = mime {
                                    let (sup_type, sub_type) =
                                        (file_mime.type_().as_str(), file_mime.subtype().as_str());

                                    if !sup_type.is_empty()
                                        && sup_type.ne("*")
                                        && item_type != sup_type
                                    {
                                        return false;
                                    }

                                    if !sub_type.is_empty()
                                        && sub_type.ne("*")
                                        && item_sub_type != sub_type
                                    {
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
                        matched_files.extend(
                            get_matched_files(entry.path().as_path().into(), jwt, mime).await?,
                        );
                    }
                }
            }
            Err(e) => {
                error!("get_matched_files() failed: {e}");
            }
        }

        Ok(matched_files)
    }

    let root = public_files_root();
    let mut files = get_matched_files(&root, &jwt, &mime).await?;

    files.shuffle(&mut thread_rng());

    match files.first() {
        Some(file_path) => {
            let file_path = &file_path.to_string_lossy()[files_root().to_string_lossy().len()..];
            let file_path: PathBuf = file_path.into();

            let uri = if preview.unwrap_or(false) {
                uri!("/api/v1/files", api_get_preview_file(path = file_path))
            } else {
                uri!("/api/v1/files", api_get_raw_file(path = file_path))
            };

            Ok(HbpResponse::redirect(uri))
        }
        None => Err(ApiError::not_found().into()),
    }
}

#[get("/preview/<path..>")]
async fn api_get_preview_file(
    jwt: Option<AuthPayload>,
    path: PathBuf,
) -> HbpResult<HbpResponse> {
    let path = files_root().join(path);

    attempt_access(&path, &jwt)?;
    assert_file_access(&path)?;

    let thumbnail = create_thumbnail(&path)?;

    Ok(HbpResponse::temp_file(thumbnail))
}

#[get("/raw/<path..>", rank = 2)]
async fn api_get_raw_file(jwt: Option<AuthPayload>, path: PathBuf) -> HbpResult<HbpResponse> {
    let path = files_root().join(path);

    attempt_access(&path, &jwt)?;
    assert_file_access(&path)?;

    Ok(HbpResponse::file(path))
}

pub fn files_api_routes() -> Vec<Route> {
    routes![
        api_get_raw_file,
        api_get_preview_file,
        api_get_random_file,
        api_get_directory
    ]
}
