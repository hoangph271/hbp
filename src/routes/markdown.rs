use crate::utils::{
    markdown,
    responders::{HbpContent, HbpResponse},
    template,
};
use anyhow::{Error, Result};
use httpstatus::StatusCode;
use mustache::MapBuilder;
use std::fs;
use std::path::Path;

#[get("/<file_name>")]
pub fn markdown_file(file_name: &str) -> HbpResponse {
    let is_markdown = file_name.to_lowercase().ends_with(".md");

    if !is_markdown {
        return HbpResponse::text("NOT a .md file", StatusCode::BadRequest);
    }

    match read_markdown(file_name) {
        Ok(content) => {
            let html_markdown = markdown::markdown_to_html(&content);
            let template_data = MapBuilder::new()
                .insert_str("raw_content", &html_markdown)
                .build();
            let html = template::render_from_template("index.html", &template_data).unwrap();

            HbpResponse::ok(Some(HbpContent::Html(html)))
        }
        Err(e) => {
            error!("{e}");

            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

fn read_markdown(file_name: &str) -> Result<String> {
    match fs::read_to_string(Path::new("markdown").join(file_name)) {
        Ok(content) => Ok(content),
        Err(e) => Err(Error::new(e)),
    }
}
