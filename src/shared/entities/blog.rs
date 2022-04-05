use crate::utils::types::{HbpError, HbpResult};
use chrono::{DateTime, Utc};
use mustache::{Data, MapBuilder};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct Blog {
    pub title: String,
    pub content: String,
    pub dob: String,
    pub tags: Option<Vec<String>>,
}

impl From<Blog> for Data {
    fn from(blog: Blog) -> Data {
        MapBuilder::new()
            .insert("title", &blog.title)
            .unwrap()
            .insert("dob", &blog.dob)
            .unwrap()
            .insert("content", &blog.content)
            .unwrap()
            .build()
    }
}

impl Blog {
    pub fn from_markdown(path: &PathBuf) -> HbpResult<Blog> {
        if !path.exists() {
            return Err(HbpError::from_message(&format!(
                "{} NOT exists",
                path.to_string_lossy()
            )));
        }

        let content = fs::read_to_string(path)?;

        if let Some(header_comment) = Regex::new("<!--((.|\n)*?)-->")?.find(&content) {
            let header_comment = &content
                [(header_comment.start() + "<!--".len())..(header_comment.end() + "-->".len())];

            let header_parts = header_comment.trim().split("\r\n");

            println!("{:?}", header_parts);

            Err(HbpError::unimplemented())
        } else {
            Ok(Blog {
                content,
                tags: None,
                title: match path.file_name() {
                    Some(file_name) => file_name.to_string_lossy().into_owned(),
                    None => path.to_string_lossy().into_owned(),
                },
                dob: format!(
                    "{}",
                    DateTime::<Utc>::from(path.metadata()?.created()?)
                        .date()
                        .format("YYYY/mm/dd")
                ),
            })
        }
    }
}
