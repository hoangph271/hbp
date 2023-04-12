use crate::shared::entities::markdown::*;
use crate::shared::interfaces::ApiError;
use crate::utils::fso::render_fso_list;
use crate::utils::responders::HbpResult;
use crate::utils::template::{IndexLayout, MoveUpUrl, Templater};

use crate::utils::{
    auth::AuthPayload,
    fso,
    responders::{HbpContent, HbpResponse},
};
use async_std::fs;
use httpstatus::StatusCode;
use log::*;
use rocket::{get, uri, State};
use serde::Serialize;
use sled::Db;
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

    if !(fso::is_markdown(&sub_path) || fso::is_plaintext(&sub_path)) {
        return if file_path.is_dir() {
            let layout_data = IndexLayout::default()
                .moveup_urls(MoveUpUrl::from_path(&file_path))
                .set_auth(jwt)
                .title(
                    &file_path
                        .file_name()
                        .map(|file_name| file_name.to_string_lossy())
                        .unwrap_or_else(|| file_path.to_string_lossy()),
                );

            render_dir(&file_path, layout_data)
        } else {
            Ok(HbpResponse::file(file_path))
        };
    }

    let markdown_data = FsoMarkdown::from_markdown(&file_path)?;

    let html = async {
        if fso::is_marp(&markdown_data.content) {
            fso::render_marp(&markdown_data).await
        } else {
            fso::render_markdown(
                &markdown_data,
                IndexLayout::default()
                    .title(&markdown_data.title)
                    .set_auth(jwt)
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

// TODO: Rename this route, not a markdown handler anymore
#[get("/users/<username>/<sub_path..>", rank = 1)]
pub(super) async fn user_markdown_file(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
    db: &State<Db>,
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
                .title(&file_path_str)
                .username(username)
                .moveup_urls(moveup_urls),
        );
    }

    if fso::is_plaintext(&file_path) {
        let raw_content = fs::read_to_string(file_path.clone()).await.unwrap();

        return Ok(HbpResponse::ok({
            let filename = file_path.file_name().unwrap().to_string_lossy();

            Some(HbpContent::Html(
                Templater::index()
                    .to_html(IndexLayout::from_title(&filename).raw_content({
                        let lines: Vec<_> = raw_content.split(' ').collect();

                        &lines.join("<br />")
                    }))
                    .unwrap(),
            ))
        }));
    }

    if fso::is_markdown(&file_path) {
        let markdown_data = FsoMarkdown::from_markdown(&file_path)?;
        let html = async {
            if fso::is_marp(&markdown_data.content) {
                fso::render_marp(&markdown_data).await
            } else {
                fso::render_user_markdown(&markdown_data, &jwt, &file_path, &db).await
            }
        }
        .await?;

        return Ok(HbpResponse::ok(Some(HbpContent::Html(html))));
    }

    Ok(HbpResponse::file(file_path))
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
    let markdowns: Vec<FsoEntry> = fso::from_dir(dir_path)?;

    render_fso_list(layout_data, markdowns).map(|html| HbpResponse::html(html, StatusCode::Ok))
}
