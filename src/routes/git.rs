use crate::shared::ApiError;
use httpstatus::StatusCode;
use rocket::{get, routes, uri, Route};
use std::{path::PathBuf, process::Command};

use crate::utils::{
    responders::{HbpError, HbpResponse, HbpResult},
    template::{IndexLayout, Templater},
};

#[get("/")]
fn git_index() -> HbpResult<HbpResponse> {
    let html =
        Templater::new("git.html".into()).to_html_page("", IndexLayout::from_title("git.html"))?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/pull")]
fn git_pull() -> HbpResult<HbpResponse> {
    let output = Command::new("git")
        .current_dir::<PathBuf>("markdown/users/hbp".into())
        .arg("pull")
        .output()
        .map_err(|e| {
            log::error!("Repository::init failed: {e:?}");
            HbpError::from(ApiError::internal_server_error())
        })?;

    log::info!("git pull: {output:?}");

    Ok(HbpResponse::redirect(uri!("/git", git_index())))
}

pub fn git_routes() -> Vec<Route> {
    routes![git_index, git_pull]
}
