use crate::utils::{
    jwt::JwtPayload,
    markdown, marper,
    responders::{HbpContent, HbpResponse},
    template,
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

#[get("/<file_path..>")]
pub async fn markdown_file(file_path: PathBuf) -> HbpResponse {
    if !is_markdown(&file_path) {
        // TODO: Maybe handle binary files as well...?
        return HbpResponse::text("NOT a .md file", StatusCode::BadRequest);
    }

    if file_path.starts_with("users") {
        // TODO: Maybe allow users with matched JWT...?
        println!("Block tis...!");
        return HbpResponse::forbidden();
    }

    let file_path = PathBuf::from("markdown").join(file_path);
    match markdown::read_markdown(&file_path) {
        Ok(content) => {
            let html = if marper::is_marp(&content) {
                let marp_content = marper::marp_from_markdown(content.to_owned()).await;

                let raw_content = [
                    marp_content.html,
                    format!("<style>{}</style>", marp_content.css),
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
                .unwrap()
            } else {
                let markdown_html = markdown::markdown_to_html(&content);
                template::render_from_template_by_default_page(
                    "static/markdown.html",
                    &Some(
                        MapBuilder::new()
                            .insert_str("markdown_html", &markdown_html)
                            .build(),
                    ),
                )
                .unwrap()
            };

            HbpResponse::ok(Some(HbpContent::Html(html)))
        }
        Err(e) => {
            error!("{e}");

            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

#[get("/users/<username>/<file_path..>")]
pub fn user_markdown_file(username: &str, file_path: PathBuf, jwt: JwtPayload) -> HbpResponse {
    if jwt.sub.eq(username) {
        let file_path = PathBuf::from("markdown")
            .join("users")
            .join(username)
            .join(file_path);

        if let Ok(content) = markdown::read_markdown(&file_path) {
            let html = markdown::markdown_to_html(&content);
            return HbpResponse::ok(Some(HbpContent::Html(html)));
        } else {
            return HbpResponse::internal_server_error();
        }
    }

    HbpResponse::status(StatusCode::Forbidden)
}
