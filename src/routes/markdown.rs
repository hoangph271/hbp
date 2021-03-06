use crate::shared::entities::markdown::*;
use crate::utils::template::{render_from_template, DefaultLayoutData};
use crate::utils::{
    auth::{AuthPayload, UserPayload},
    markdown,
    responders::{HbpContent, HbpResponse},
    string::url_encode_path,
};
use httpstatus::StatusCode;
use log::*;
use mustache::Data;
use rocket::{get, routes, Route};
use std::path::{Path, PathBuf};

fn assert_payload_access(payload: &UserPayload, path: &str) -> bool {
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

#[get("/<sub_path..>", rank = 2)]
async fn markdown_file(sub_path: PathBuf, jwt: Option<AuthPayload>) -> HbpResponse {
    let file_path = PathBuf::from("markdown").join(sub_path.clone());

    if !markdown::is_markdown(&sub_path) {
        return HbpResponse::file(file_path);
    }

    match Markdown::from_markdown(&file_path) {
        Ok(markdown_data) => {
            let html_result = async {
                if markdown::is_marp(&markdown_data.content) {
                    markdown::render_marp(&markdown_data, markdown_extra_data(&file_path)).await
                } else {
                    let html = markdown::render_markdown(
                        &markdown_data,
                        Some(DefaultLayoutData::only_title(&markdown_data.title).maybe_auth(jwt)),
                    )
                    .await;
                    html
                }
            };

            match html_result.await {
                Ok(html) => HbpResponse::html(&html, None),
                Err(e) => {
                    error!("{}", e);
                    HbpResponse::status(StatusCode::InternalServerError)
                }
            }
        }
        Err(e) => {
            error!("{e}");
            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

#[get("/_edit/<sub_path..>")]
async fn user_markdown_editor(sub_path: PathBuf, _jwt: AuthPayload) -> HbpResponse {
    let _file_path_str = PathBuf::from("markdown").join(sub_path);

    HbpResponse::html(
        &render_from_template("markdown/write-markdown.html", None).unwrap(),
        None,
    )
}

fn moveup_url_from(file_path: &Path) -> String {
    let moveup_url = if let Some(parent_path) = file_path.parent() {
        let parent_path = parent_path.to_string_lossy().to_string();
        let is_user_root = parent_path.eq("markdown/users");

        if is_user_root {
            "".to_string()
        } else {
            parent_path
        }
    } else {
        "".to_string()
    };

    url_encode_path(&moveup_url)
}

#[get("/users/<username>/<sub_path..>", rank = 1)]
async fn user_markdown_file(username: &str, sub_path: PathBuf, jwt: AuthPayload) -> HbpResponse {
    let (file_path_str, file_path) = markdown_path_from(username, &sub_path);

    if !jwt.match_path(&file_path_str, Some(assert_payload_access)) {
        return HbpResponse::forbidden();
    }

    if !file_path.exists() {
        info!("{:?} not exists", file_path.to_string_lossy());
        return HbpResponse::not_found();
    }

    let moveup_url = moveup_url_from(&file_path);

    if file_path.is_dir() {
        let markdowns: Vec<MarkdownOrMarkdownDir> =
            markdown::markdown_from_dir(&file_path).unwrap();

        return HbpResponse::html(
            &markdown::render_markdown_list(
                DefaultLayoutData::only_title(&file_path_str)
                    .username(username)
                    .moveup_url(&moveup_url),
                markdowns,
            ),
            None,
        );
    }

    if !markdown::is_markdown(&file_path) {
        return if file_path.is_dir() {
            let mut markdowns = markdown::markdown_from_dir(&file_path).unwrap();

            markdowns.iter_mut().for_each(|markdown| {
                if let MarkdownOrMarkdownDir::Markdown(markdown) = markdown {
                    if markdown.author.is_empty() {
                        markdown.author = username.to_owned();
                    }
                }
            });

            // TODO: Sort...! :"<
            // markdowns.sort_by(|m1, m2| {
            //     const DATE_FORMAT: &str = "%m/%d/%Y";
            //     NaiveDate::parse_from_str(&m2.dob, DATE_FORMAT)
            //         .unwrap()
            //         .cmp(&NaiveDate::parse_from_str(&m1.dob, DATE_FORMAT).unwrap())
            // });

            HbpResponse::html(
                &markdown::render_markdown_list(
                    DefaultLayoutData::only_title(&file_path_str)
                        .maybe_auth(Some(jwt))
                        .moveup_url(&moveup_url),
                    markdowns,
                ),
                None,
            )
        } else {
            HbpResponse::file(file_path)
        };
    }

    if let Ok(markdown_data) = Markdown::from_markdown(&file_path) {
        let html_result = async {
            if markdown::is_marp(&markdown_data.content) {
                markdown::render_marp(&markdown_data, markdown_extra_data(&file_path)).await
            } else {
                markdown::render_markdown(
                    &markdown_data,
                    Some(
                        DefaultLayoutData::only_title(&markdown_data.title)
                            .maybe_auth(Some(jwt))
                            .moveup_url(&moveup_url),
                    ),
                )
                .await
            }
        }
        .await;

        match html_result {
            Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
            Err(e) => {
                error!("{}", e);
                HbpResponse::internal_server_error()
            }
        }
    } else {
        HbpResponse::internal_server_error()
    }
}

fn markdown_extra_data(file_path: &Path) -> Option<Vec<(String, Data)>> {
    if let Ok(markdown) = Markdown::from_markdown(file_path) {
        Some(vec![
            ("title".to_owned(), Data::String(markdown.title.clone())),
            ("og_title".to_owned(), Data::String(markdown.title)),
            ("og_image".to_owned(), Data::String(markdown.cover_image)),
        ])
    } else {
        None
    }
}

pub fn markdown_routes() -> Vec<Route> {
    routes![markdown_file, user_markdown_file, user_markdown_editor,]
}
