use crate::utils::{
    jwt::JwtPayload,
    markdown, marper,
    responders::{HbpContent, HbpResponse},
    template,
    types::HbpResult,
};
use httpstatus::StatusCode;
use mustache::{Data, MapBuilder};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct MarkdownOgpMetadata {
    og_title: String,
    og_type: String,
    og_url: String,
    og_image: String,
}
impl MarkdownOgpMetadata {
    fn of_markdown(markdown_path: &Path) -> Option<MarkdownOgpMetadata> {
        let json_file_name = match markdown_path.file_name() {
            Some(file_name) => {
                let mut file_name = file_name.to_string_lossy().into_owned();
                file_name.push_str(".json");

                file_name
            }
            None => return None,
        };

        let mut json_path = markdown_path.to_owned();
        json_path.set_file_name(json_file_name);

        if json_path.exists() {
            if let Ok(mut file) = File::open(json_path) {
                let mut json = String::new();

                if file.read_to_string(&mut json).is_err() {
                    return None;
                }

                if let Ok(json) = serde_json::from_str::<MarkdownOgpMetadata>(&json) {
                    return Some(json);
                }

                debug!(
                    "is_err: {:?}",
                    serde_json::from_str::<MarkdownOgpMetadata>(&json)
                );
            }
        }

        None
    }

    fn to_data(&self) -> Data {
        MapBuilder::new()
            .insert_str("og_title", self.og_title.clone())
            .insert_str("og_type", self.og_type.clone())
            .insert_str("og_url", self.og_url.clone())
            .insert_str("og_image", self.og_image.clone())
            .build()
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

    let mut data = vec![("raw_content".to_owned(), Data::String(raw_content))];

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
                Data::String(markdown_html),
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
                    Data::String(file_path.to_string_lossy().into_owned()),
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

    if !sub.eq(username) {
        return HbpResponse::status(StatusCode::Forbidden);
    }

    let markdown_path = PathBuf::from("markdown")
        .join("users")
        .join(username)
        .join(file_path);

    let title = match markdown_path.file_name() {
        Some(file_name) => file_name.to_string_lossy().into_owned(),
        None => markdown_path.to_string_lossy().into_owned(),
    };

    let ogp_metadata = MarkdownOgpMetadata::of_markdown(&markdown_path);

    let extra_data = vec![
        ("title".to_owned(), Data::String(title)),
        (
            "ogp_metadata".to_owned(),
            match ogp_metadata {
                Some(ogp_metadata) => ogp_metadata.to_data(),
                None => Data::Null,
            },
        ),
    ];

    if !markdown_path.exists() {
        return HbpResponse::not_found()
    }

    if !is_markdown(&markdown_path) {
        // TODO: Maybe handle binary files as well...?
        return HbpResponse::text("NOT a .md file", StatusCode::BadRequest);
    }

    if let Ok(content) = markdown::read_markdown(&markdown_path) {
        return match render_markdown(&content, Some(extra_data)).await {
            Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
            Err(e) => {
                error!("{}", e);
                HbpResponse::internal_server_error()
            }
        };
    }

    HbpResponse::internal_server_error()
}
