use crate::utils::marper;
use crate::utils::template;
use crate::utils::types::HbpResult;
use anyhow::{Error, Result};
use mustache::Data;
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::Path;

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
pub async fn render_markdown(
    markdown: &str,
    extra_data: Option<template::TemplateData>,
) -> HbpResult<String> {
    if marper::is_marp(markdown) {
        marper::render_marp(markdown, extra_data).await
    } else {
        let markdown_html = markdown_to_html(markdown);

        template::render_from_template_by_default_page(
            "static/markdown.html",
            &Some(template::data_from(vec![(
                "markdown_html".to_owned(),
                Data::String(markdown_html),
            )])),
        )
    }
}
