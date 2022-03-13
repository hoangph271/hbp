use crate::utils::{
    jwt::JwtPayload,
    markdown, marper,
    responders::{HbpContent, HbpResponse},
    template,
    types::HbpResult,
};
use httpstatus::StatusCode;
use std::path::{Path, PathBuf};

fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

async fn render_marp(
    markdown: &str,
    extra_data: Option<template::TemplateData>,
) -> HbpResult<String> {
    let marp_content = marper::marp_from_markdown(markdown.to_owned()).await;

    let raw_content = [
        marp_content.html,
        format!(
            "<style>
            {css}
            .nav-bar {{
                display: none;
            }}
        </style>",
            css = marp_content.css
        ),
    ]
    .join("\n");

    let mut data = vec![("raw_content".to_owned(), raw_content)];

    if let Some(extra_data) = extra_data {
        data.extend(extra_data);
    }

    template::render_from_template("index.html", &Some(template::data_from(data)))
}
async fn render_markdown(
    markdown: &str,
    extra_data: Option<template::TemplateData>,
) -> HbpResult<String> {
    if marper::is_marp(markdown) {
        render_marp(markdown, extra_data).await
    } else {
        let markdown_html = markdown::markdown_to_html(markdown);

        template::render_from_template_by_default_page(
            "static/markdown.html",
            &Some(template::data_from(vec![(
                "markdown_html".to_owned(),
                markdown_html,
            )])),
        )
    }
}

#[get("/<file_path..>")]
pub async fn markdown_file(file_path: PathBuf) -> HbpResponse {
    if !is_markdown(&file_path) {
        // TODO: Maybe handle binary files as well...?
        return HbpResponse::text("NOT a .md file", StatusCode::BadRequest);
    }

    let file_path = PathBuf::from("markdown").join(file_path);
    match markdown::read_markdown(&file_path) {
        Ok(content) => {
            match render_markdown(
                &content,
                Some(vec![(
                    "title".to_owned(),
                    file_path.to_string_lossy().into_owned(),
                )]),
            )
            .await
            {
                Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
                Err(_) => HbpResponse::internal_server_error(),
            }
        }
        Err(e) => {
            error!("{e}");

            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

#[get("/users/<username>/<file_path..>")]
pub async fn user_markdown_file(
    username: &str,
    file_path: PathBuf,
    jwt: JwtPayload,
) -> HbpResponse {
    let sub = JwtPayload::sub_from(jwt);

    if sub.eq(username) {
        let file_path = PathBuf::from("markdown")
            .join("users")
            .join(username)
            .join(file_path);

        if let Ok(content) = markdown::read_markdown(&file_path) {
            return match render_markdown(
                &content,
                file_path.file_name().map(|file_name| {
                    vec![("title".to_owned(), file_name.to_string_lossy().into_owned())]
                }),
            )
            .await
            {
                Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
                Err(e) => {
                    error!("{}", e);
                    HbpResponse::internal_server_error()
                }
            };
        } else {
            return HbpResponse::internal_server_error();
        }
    }

    HbpResponse::status(StatusCode::Forbidden)
}
