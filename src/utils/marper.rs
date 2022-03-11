use crate::utils::env::{from_env, EnvKey};
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
