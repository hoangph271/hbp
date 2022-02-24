use crate::utils::{
    markdown,
    responders::{HbpContent, HbpResponse},
};
use anyhow::{Error, Result};
use httpstatus::StatusCode;
use std::fs;
use std::path::Path;

#[get("/<file_name>")]
pub fn markdown_file(file_name: &str) -> HbpResponse {
    let is_markdown = file_name.to_lowercase().ends_with(".md");

    if !is_markdown {
        return HbpResponse::status_text(StatusCode::BadRequest, "NOT a .md file");
    }

    match read_markdown(file_name) {
        Ok(content) => HbpResponse::ok(HbpContent::Html(markdown::markdown_to_html(&content))),
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
