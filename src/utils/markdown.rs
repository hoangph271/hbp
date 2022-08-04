use crate::shared::entities::markdown::*;
use crate::utils::marper;
use crate::utils::string::url_encode_path;
use crate::utils::template::TemplateRenderer;
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode::BadRequest;
use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;

use super::template::IndexLayoutData;

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

pub fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

pub async fn render_marp(markdown: &Markdown) -> HbpResult<String> {
    if !marper::is_marp(&markdown.content) {
        return Err(HbpError::from_message(
            &format!("NOT a marp: {}", markdown.file_name),
            BadRequest,
        ));
    }

    marper::render_marp(&markdown.content).await
}
pub fn is_marp(content: &str) -> bool {
    marper::is_marp(content)
}
pub async fn render_markdown(
    markdown: &Markdown,
    layout_data: IndexLayoutData,
) -> HbpResult<String> {
    #[derive(Serialize, Debug)]
    struct RenderData {
        markdown_html: String,
    }

    TemplateRenderer::new("markdown/markdown.html".into()).to_html_page(
        RenderData {
            markdown_html: markdown_to_html(&markdown.content),
        },
        layout_data,
    )
}

pub fn markdown_from_dir<P: AsRef<Path>>(path: &P) -> HbpResult<Vec<MarkdownOrMarkdownDir>> {
    let markdowns = read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let title = match entry.path().file_name() {
                Some(file_name) => file_name.to_string_lossy().to_string(),
                None => "Untitled".to_owned(),
            };

            if title.starts_with('.') {
                return None;
            }

            if entry.path().is_dir() {
                let path: String = entry.path().to_string_lossy().to_string();
                let url = url_encode_path(&path);

                Some(MarkdownOrMarkdownDir::MarkdownDir(MarkdownDir {
                    title,
                    url,
                }))
            } else if entry.path().to_string_lossy().ends_with(".md") {
                Some(MarkdownOrMarkdownDir::Markdown(
                    Markdown::from_markdown(&entry.path()).ok()?,
                ))
            } else {
                None
            }
        })
        .collect();

    Ok(markdowns)
}

pub fn render_markdown_list(
    layout_data: IndexLayoutData,
    markdowns: Vec<MarkdownOrMarkdownDir>,
) -> HbpResult<String> {
    let mut render_data = HashMap::new();
    render_data.insert("markdowns", markdowns);

    TemplateRenderer::new("markdown/list.html".into()).to_html_page(render_data, layout_data)
}
