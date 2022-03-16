use crate::utils::env::{from_env, EnvKey};
use crate::utils::marper;
use crate::utils::template;
use crate::utils::types::HbpResult;
use mustache::Data;
use reqwest::{multipart, Client};
use serde::Deserialize;

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

pub async fn render_marp(
    markdown: &str,
    extra_data: Option<template::TemplateData>,
) -> HbpResult<String> {
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

    let mut data = vec![("raw_content".to_owned(), Data::String(raw_content))];

    if let Some(extra_data) = extra_data {
        data.extend(extra_data);
    }

    template::render_from_template("index.html", &Some(template::data_from(data)))
}
