use crate::shared::entities::markdown::*;
use crate::shared::interfaces::ApiError;
use crate::utils::markdown::render_markdown_list;
use crate::utils::responders::HbpResult;
use crate::utils::template::{IndexLayout, MoveUpUrl, Templater};
// use crate::utils::types::HbpResult;
use crate::utils::{
    auth::AuthPayload,
    markdown,
    responders::{HbpContent, HbpResponse},
};
use httpstatus::StatusCode;
use log::*;
use rocket::{get, uri};
use serde::Serialize;
use std::path::PathBuf;

use super::{assert_payload_access, markdown_path_from};

#[get("/<sub_path..>", rank = 2)]
pub(super) async fn markdown_file(
    sub_path: PathBuf,
    jwt: Option<AuthPayload>,
) -> HbpResult<HbpResponse> {
    let file_path = PathBuf::from("markdown").join(sub_path.clone());

    if !file_path.exists() {
        return Err(ApiError::not_found().with_ui().into());
    }

    if !markdown::is_markdown(&sub_path) {
        return if file_path.is_dir() {
            let layout_data = IndexLayout::default()
                .moveup_urls(MoveUpUrl::from_path(&file_path))
                .maybe_auth(jwt)
                .title(
                    file_path
                        .file_name()
                        .map(|file_name| file_name.to_string_lossy())
                        .unwrap_or_else(|| file_path.to_string_lossy())
                        .to_string(),
                );

            render_dir(&file_path, layout_data)
        } else {
            Ok(HbpResponse::file(file_path))
        };
    }

    let markdown_data = Markdown::from_markdown(&file_path)?;

    let html = async {
        if markdown::is_marp(&markdown_data.content) {
            markdown::render_marp(&markdown_data).await
        } else {
            markdown::render_markdown(
                &markdown_data,
                IndexLayout::default()
                    .title(markdown_data.title.to_owned())
                    .maybe_auth(jwt)
                    .moveup_urls(MoveUpUrl::from_path(&file_path)),
            )
            .await
        }
    }
    .await?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/_edit/<sub_path..>")]
pub(super) async fn user_markdown_editor(sub_path: PathBuf, _jwt: AuthPayload) -> HbpResponse {
    let _file_path_str = PathBuf::from("markdown").join(sub_path);

    match Templater::new("markdown/write-markdown.html".into()).to_html(()) {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}

#[get("/users/<username>/<sub_path..>", rank = 1)]
pub(super) async fn user_markdown_file(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
) -> HbpResult<HbpResponse> {
    jwt.assert_username(username)?;

    let (file_path_str, file_path) = markdown_path_from(username, &sub_path);

    jwt.match_path(&file_path, assert_payload_access)?;

    if !file_path.exists() {
        info!("{:?} not exists", file_path.to_string_lossy());
        return Ok(HbpResponse::not_found());
    }

    let moveup_urls = MoveUpUrl::from_path(&file_path);

    if file_path.is_dir() {
        return render_dir(
            &file_path,
            IndexLayout::default()
                .title(file_path_str)
                .username(username)
                .moveup_urls(moveup_urls),
        );
    }

    if !markdown::is_markdown(&file_path) {
        return if file_path.is_dir() {
            let mut markdowns = markdown::markdown_from_dir(&file_path)?;

            markdowns.iter_mut().for_each(|markdown| {
                if let MarkdownOrMarkdownDir::Markdown(markdown) = markdown {
                    if markdown.author.is_empty() {
                        markdown.author = username.to_owned();
                    }
                }
            });

            // TODO: Sort...! :"<

            let html = render_markdown_list(
                IndexLayout::default()
                    .title(file_path_str)
                    .maybe_auth(Some(jwt))
                    .moveup_urls(moveup_urls),
                markdowns,
            )?;

            Ok(HbpResponse::html(html, StatusCode::Ok))
        } else {
            Ok(HbpResponse::file(file_path))
        };
    }

    if let Ok(markdown_data) = Markdown::from_markdown(&file_path) {
        let html = async {
            if markdown::is_marp(&markdown_data.content) {
                markdown::render_marp(&markdown_data).await
            } else {
                markdown::render_user_markdown(&markdown_data, &jwt, &file_path).await
            }
        }
        .await?;

        Ok(HbpResponse::ok(Some(HbpContent::Html(html))))
    } else {
        Ok(HbpResponse::internal_server_error())
    }
}

#[get("/users", rank = 1)]
pub(super) async fn user_default(jwt: AuthPayload) -> HbpResponse {
    let uri = uri!(
        "/markdown",
        user_markdown_file(jwt.username(), PathBuf::new())
    );

    HbpResponse::redirect(uri)
}

#[derive(Serialize, Default)]
pub struct MarkdownExtraData {
    title: String,
    og_title: String,
    og_image: String,
}

fn render_dir(dir_path: &PathBuf, layout_data: IndexLayout) -> HbpResult<HbpResponse> {
    let markdowns: Vec<MarkdownOrMarkdownDir> = markdown::markdown_from_dir(dir_path)?;

    render_markdown_list(layout_data, markdowns).map(|html| HbpResponse::html(html, StatusCode::Ok))
}
