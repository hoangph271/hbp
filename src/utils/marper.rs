use crate::routes::markdown::MarkdownExtraData;
use crate::utils::env::{from_env, EnvKey};
use crate::utils::marper;
use crate::utils::types::HbpResult;
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};

use super::template::TemplateRenderer;

#[derive(Deserialize, Debug)]
pub struct MarpContent {
    pub html: String,
    pub css: String,
}

pub fn is_marp(markdown: &str) -> bool {
    let parts: Vec<&str> = markdown.split("---").collect();

    match parts.get(1) {
        Some(header) => {
            return header.trim().contains("marp: true");
        }
        None => false,
    }
}

pub async fn marp_from_markdown(markdown: String) -> MarpContent {
    let url = format!("{}?json=1", from_env(EnvKey::MarpApiRoot));
    let form = multipart::Form::new().text("markdown", markdown);

    let res = Client::new()
        .post(url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    res.json::<MarpContent>().await.unwrap()
}

pub async fn render_marp(markdown: &str, extra_data: MarkdownExtraData) -> HbpResult<String> {
    let marp_content = marper::marp_from_markdown(markdown.to_owned()).await;

    let raw_content = [
        marp_content.html,
        format!(
            "<style>
            {css}
            .nav-bar {{
                display: none;
            }}
        </style>",
            css = marp_content.css
        ),
    ]
    .join("\n");

    #[derive(Serialize)]
    struct RenderData {
        raw_content: String,
        extra_data: MarkdownExtraData,
    }

    TemplateRenderer::new("index.html".into()).to_html(RenderData {
        raw_content,
        extra_data,
    })
}
