use crate::shared::entities::markdown::*;
use crate::utils::marper;
use crate::utils::template::{
    render_default_layout, simple_data_from, DefaultLayoutData, TemplateData,
};
use crate::utils::types::{HbpError, HbpResult};
use mustache::Data;
use mustache::MapBuilder;
use pulldown_cmark::{html, Options, Parser};
use std::fs::read_dir;
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

pub fn is_markdown(file_path: &Path) -> bool {
    match file_path.file_name() {
        None => false,
        Some(file_name) => file_name.to_string_lossy().to_lowercase().ends_with(".md"),
    }
}

pub async fn render_marp(
    markdown: &Markdown,
    extra_data: Option<TemplateData>,
) -> HbpResult<String> {
    if !marper::is_marp(&markdown.content) {
        return Err(HbpError::from_message(&format!(
            "NOT a marp: {}",
            markdown.file_name
        )));
    }

    marper::render_marp(&markdown.content, extra_data).await
}
pub fn is_marp(content: &str) -> bool {
    marper::is_marp(content)
}
pub async fn render_markdown(
    markdown: &Markdown,
    layout_data: Option<DefaultLayoutData>,
) -> HbpResult<String> {
    let markdown_html = markdown_to_html(&markdown.content);

    render_default_layout(
        "markdown/markdown.html",
        layout_data,
        Some(simple_data_from(vec![(
            "markdown_html".to_owned(),
            Data::String(markdown_html),
        )])),
    )
}

pub fn markdown_from_dir<P: AsRef<Path>>(path: &P) -> Vec<MarkdownOrMarkdownDir> {
    read_dir(path)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();

            if entry.path().is_dir() {
                Some(MarkdownOrMarkdownDir::MarkdownDir(MarkdownDir {
                    title: match entry.path().file_name() {
                        Some(file_name) => file_name.to_string_lossy().to_string(),
                        None => "Untitled".to_owned(),
                    },
                    url: format!("{}", entry.path().to_string_lossy()),
                }))
            } else if entry.path().to_string_lossy().ends_with(".md") {
                Some(MarkdownOrMarkdownDir::Markdown(
                    Markdown::from_markdown(&entry.path()).unwrap(),
                ))
            } else {
                None
            }
        })
        .collect()
}

pub fn render_markdown_list(
    default_layout_data: DefaultLayoutData,
    markdowns: Vec<MarkdownOrMarkdownDir>,
) -> String {
    render_default_layout(
        "markdown/list.html",
        Some(default_layout_data),
        Some(
            MapBuilder::new()
                .insert_vec("markdowns", |builder| {
                    let mut builder = builder;

                    for markdown in &markdowns {
                        builder = match markdown {
                            MarkdownOrMarkdownDir::Markdown(markdown) => {
                                builder.push(&markdown).unwrap()
                            }
                            MarkdownOrMarkdownDir::MarkdownDir(markdown_dir) => {
                                builder.push(&markdown_dir).unwrap()
                            }
                        };
                    }

                    builder
                })
                .build(),
        ),
    )
    .unwrap()
}
