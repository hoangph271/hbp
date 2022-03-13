use crate::utils::{
    jwt::JwtPayload,
    markdown, marper,
    responders::{HbpContent, HbpResponse},
    template,
    types::HbpResult,
};
use httpstatus::StatusCode;
use mustache::MapBuilder;
use std::path::{Path, PathBuf};

fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

async fn render_marp(markdown: &str) -> HbpResult<String> {
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

    template::render_from_template(
        "index.html",
        &Some(
            MapBuilder::new()
                .insert_str("raw_content", raw_content)
                .build(),
        ),
    )
}
async fn render_markdown(markdown: &str) -> HbpResult<String> {
    if marper::is_marp(markdown) {
        render_marp(markdown).await
    } else {
        let markdown_html = markdown::markdown_to_html(markdown);

        template::render_from_template_by_default_page(
            "static/markdown.html",
            &Some(
                MapBuilder::new()
                    .insert_str("markdown_html", &markdown_html)
                    .build(),
            ),
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
        Ok(content) => match render_markdown(&content).await {
            Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
            Err(_) => HbpResponse::internal_server_error(),
        },
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
            return match render_markdown(&content).await {
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
