use crate::shared::entities::blog::MarkdownData;
use crate::utils::marper;
use crate::utils::template::{
    render_default_layout, simple_data_from, DefaultLayoutData, TemplateData,
};
use crate::utils::types::HbpResult;
use mustache::Data;
use pulldown_cmark::{html, Options, Parser};
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
