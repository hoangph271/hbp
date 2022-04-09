use crate::utils::types::{HbpError, HbpResult};
use anyhow::Result;
use chrono::{DateTime, Utc};
use mustache::{Data, EncoderError, MapBuilder};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Default)]
pub struct Markdown {
    pub title: String,
    pub file_name: String,
    pub author: String,
    pub content: String,
    pub dob: String,
    pub tags: Option<Vec<String>>,
    pub cover_image: String,
}

impl From<Markdown> for Data {
    fn from(markdown: Markdown) -> Data {
        let insert_fields = move || -> Result<Data, EncoderError> {
            let mut map_builder = MapBuilder::new()
                .insert("title", &markdown.title)?
                .insert("content", &markdown.content)?
                .insert("file_name", &markdown.file_name)?
                .insert("author", &markdown.author)?
                .insert("cover_image", &markdown.cover_image)?
                .insert("dob", &markdown.dob)?;

            if let Some(tags) = markdown.tags {
                map_builder = map_builder.insert_vec("tags", |mut builder| {
                    for tag in &tags[..] {
                        builder = builder.push(tag).unwrap();
                    }

                    builder
                })
            }

            Ok(map_builder.build())
        };

        insert_fields().unwrap()
    }
}

fn extract_markdown_header_content(content: &str) -> Option<String> {
    if let Some(header_comment) = Regex::new("<!--((.|\n)*?)-->").ok()?.find(content) {
        if header_comment.start() != 0 {
            None
        } else {
            let (start, end) = (
                header_comment.start() + "<!--".len(),
                header_comment.end() - "-->".len(),
            );
            let header_content = (&content[start..end]).to_string();
            Some(header_content)
        }
    } else {
        None
    }
}

impl Markdown {
    pub fn from_markdown(path: &Path) -> HbpResult<Markdown> {
        if !path.exists() {
            return Err(HbpError::from_message(&format!(
                "{} NOT exists",
                path.to_string_lossy()
            )));
        }

        let mut markdown = Markdown {
            content: fs::read_to_string(path)?,
            file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
            ..Markdown::default()
        };

        if let Some(header_comment) = extract_markdown_header_content(&markdown.content) {
            let mut header_map: HashMap<String, String> = header_comment
                .trim()
                .split('\n')
                .map(|line| {
                    let colon_index = line.find(':').unwrap();

                    (
                        (&line[..colon_index]).trim().to_string(),
                        (&line[colon_index + 1..]).trim().to_string(),
                    )
                })
                .collect();

            markdown.title = if let Some(title) = header_map.remove("title") {
                title
            } else {
                markdown.file_name.clone()
            };

            if let Some(author) = header_map.remove("author") {
                markdown.author = author;
            }

            if let Some(tags) = header_map.remove("tags") {
                markdown.tags = Some(tags.split(',').map(|tag| tag.trim().to_owned()).collect());
            }

            if let Some(cover_image) = header_map.remove("cover_image") {
                markdown.cover_image = cover_image;
            }

            markdown.dob = if let Some(dob) = header_map.remove("dob") {
                dob
            } else {
                format!(
                    "{}",
                    DateTime::<Utc>::from(path.metadata()?.created()?)
                        .date()
                        .format("YYYY/mm/dd")
                )
            }
        }

        Ok(markdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skip_if_no_metadata_comment() {
        assert_eq!(extract_markdown_header_content(""), None);
    }
}
