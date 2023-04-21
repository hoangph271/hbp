use crate::data::tiny_url_orm::TinyUrlOrm;
use crate::shared::entities::markdown::*;
use crate::shared::interfaces::ApiError;
use crate::utils::template::Templater;
use httpstatus::StatusCode::BadRequest;
use log::error;
use pulldown_cmark::{html, Options, Parser};

use sled::Db;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;


use super::auth::{AuthPayload};
use super::marper;
use super::responders::HbpResult;
use super::template::{IndexLayout, MarkdownTemplate, MoveUpUrl};

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
pub fn is_plaintext(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".txt"),
    }
}

pub async fn render_marp(markdown: &FsoMarkdown) -> HbpResult<String> {
    if !marper::is_marp(&markdown.content) {
        return Err(ApiError::from_message(
            &format!("NOT a marp: {}", markdown.file_name),
            BadRequest,
        )
        .into());
    }

    marper::render_marp(&markdown.content).await
}
pub fn is_marp(content: &str) -> bool {
    marper::is_marp(content)
}
pub async fn render_markdown(
    markdown: &FsoMarkdown,
    layout_data: IndexLayout,
) -> HbpResult<String> {
    Templater::new("markdown/markdown.html".into())
        .to_html_page(MarkdownTemplate::of(markdown, None), layout_data)
}

pub fn allowed_glob(file_path: &Path) -> String {
    if file_path.is_file() {
        let file_stem = file_path.file_stem().map(|val| val.to_string_lossy());
        let parent_path = file_path.parent();

        if let (Some(file_stem), Some(parent_path)) = (file_stem, parent_path) {
            if parent_path.ends_with(file_stem.to_string()) {
                let parent_glob = parent_path.join("*");
                return parent_glob.to_string_lossy().to_string();
            }
        }
    }

    file_path.to_string_lossy().to_string()
}

pub async fn render_user_markdown(
    markdown: &FsoMarkdown,
    jwt: &AuthPayload,
    file_path: &Path,
    db: &Db,
) -> HbpResult<String> {
    let layout_data = IndexLayout::default()
        .title(&markdown.title)
        .username(jwt.username())
        .moveup_urls(MoveUpUrl::from_path(file_path));

    let signed_url = {
        TinyUrlOrm::default()
            .find_one_by_full_url(db, &markdown.url)
            .await
            .unwrap()
            .map(|tiny_url| {
                tiny_url.get_full_url()
            })
    };

    Templater::new("markdown/markdown.html".into())
        .to_html_page(MarkdownTemplate::of(markdown, signed_url), layout_data)
}

pub fn from_dir<P: AsRef<Path>>(path: &P) -> HbpResult<Vec<FsoEntry>> {
    let markdowns = read_dir(path)
        .map_err(|e| {
            error!("read_dir failed: {e:?}");
            ApiError::internal_server_error()
        })?
        .filter_map(|entry| {
            let entry = entry.ok()?;

            if entry.file_name().to_string_lossy().starts_with('.') {
                return None;
            }

            Some(FsoEntry::from_path(&entry.path()))
        })
        .collect();

    Ok(markdowns)
}

pub fn render_fso_list(layout_data: IndexLayout, markdowns: Vec<FsoEntry>) -> HbpResult<String> {
    let mut render_data = HashMap::new();
    render_data.insert("markdowns", markdowns);

    Templater::new("markdown/list.html".into()).to_html_page(render_data, layout_data)
}
