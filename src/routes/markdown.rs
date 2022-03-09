use crate::utils::{
    jwt::JwtPayload,
    markdown,
    responders::{HbpContent, HbpResponse},
    template,
};
use httpstatus::StatusCode;
use mustache::MapBuilder;
use std::path::PathBuf;

#[get("/<file_path..>")]
pub fn markdown_file(file_path: PathBuf) -> HbpResponse {
    match file_path.file_name() {
        Some(file_name) => {
            let is_markdown = file_name.to_string_lossy().to_lowercase().ends_with(".md");

            if !is_markdown {
                // TODO: Maybe handle binary files as well...?
                return HbpResponse::text("NOT a .md file", StatusCode::BadRequest);
            }
        }
        None => return HbpResponse::text("NOT a .md file", StatusCode::BadRequest),
    }

    if file_path.starts_with("users") {
        println!("Block tis...!");
    }

    let file_path = PathBuf::from("markdown").join(file_path);
    match markdown::read_markdown(&file_path) {
        Ok(content) => {
            let markdown_html = markdown::markdown_to_html(&content);

            let html = template::render_from_template_paged(
                "static/markdown.html",
                &Some(
                    MapBuilder::new()
                        .insert_str("markdown_html", &markdown_html)
                        .build(),
                ),
            )
            .unwrap();
            println!("{html}");

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

        println!("{}", file_path.to_string_lossy());

        if let Ok(content) = markdown::read_markdown(&file_path) {
            let html = markdown::markdown_to_html(&content);
            return HbpResponse::ok(Some(HbpContent::Html(html)));
        } else {
            return HbpResponse::internal_server_error();
        }
    }

    HbpResponse::status(StatusCode::Forbidden)
}
