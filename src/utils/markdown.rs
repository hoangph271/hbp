use crate::utils::marper;
use crate::utils::template::{
    render_default_layout, simple_data_from, DefaultLayoutData, TemplateData,
};
use crate::utils::types::HbpResult;
use anyhow::{Error, Result};
use mustache::Data;
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::{Path, PathBuf};

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    html
}

pub fn read_markdown(file_path: &Path) -> Result<String> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(Error::new(e)),
    }
}

pub fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

pub struct MarkdownData {
    content: String,
    file_path: PathBuf,
}
impl MarkdownData {
    pub fn from_file(file_path: PathBuf) -> HbpResult<MarkdownData> {
        let content = read_markdown(&file_path)?;

        Ok(MarkdownData { content, file_path })
    }

    pub fn title(&self) -> String {
        if let Some(title) = self.file_path.file_name() {
            title.to_string_lossy().into_owned()
        } else {
            self.file_path.to_string_lossy().into_owned()
        }
    }
}

pub async fn render_markdown(
    markdown_data: &MarkdownData,
    extra_data: Option<TemplateData>,
) -> HbpResult<String> {
    if marper::is_marp(&markdown_data.content) {
        marper::render_marp(&markdown_data.content, extra_data).await
    } else {
        let markdown_html = markdown_to_html(&markdown_data.content);

        render_default_layout(
            "static/markdown.html",
            Some(DefaultLayoutData::only_title(&markdown_data.title())),
            Some(simple_data_from(vec![(
                "markdown_html".to_owned(),
                Data::String(markdown_html),
            )])),
        )
    }
}
