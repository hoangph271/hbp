use crate::utils::types::{HbpError, HbpResult};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use mustache::{Data, EncoderError, MapBuilder};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Default)]
pub struct Blog {
    pub title: String,
    pub file_name: String,
    pub author: String,
    pub content: String,
    pub dob: String,
    pub tags: Option<Vec<String>>,
    pub cover_image: String,
}

impl From<Blog> for Data {
    fn from(blog: Blog) -> Data {
        let insert_fields = move || -> Result<Data, EncoderError> {
            let mut map_builder = MapBuilder::new()
                .insert("title", &blog.title)?
                .insert("content", &blog.content)?
                .insert("file_name", &blog.file_name)?
                .insert("author", &blog.author)?
                .insert("cover_image", &blog.cover_image)?
                .insert("dob", &blog.dob)?;

            if let Some(tags) = blog.tags {
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

impl Blog {
    pub fn from_markdown(path: &Path) -> HbpResult<Blog> {
        if !path.exists() {
            return Err(HbpError::from_message(&format!(
                "{} NOT exists",
                path.to_string_lossy()
            )));
        }

        let content = fs::read_to_string(path)?;
        let mut blog = Blog {
            content: content.clone(),
            file_name: path.file_name().unwrap().to_string_lossy().into_owned(),
            ..Blog::default()
        };

        if let Some(header_comment) = Regex::new("<!--((.|\n)*?)-->")?.find(&content) {
            let header_comment = &content
                [(header_comment.start() + "<!--".len())..(header_comment.end() - "-->".len())];

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

            blog.title = if let Some(title) = header_map.remove("title") {
                title.to_owned()
            } else {
                blog.file_name.clone()
            };

            if let Some(author) = header_map.remove("author") {
                blog.author = author.to_owned();
            }

            if let Some(tags) = header_map.remove("tags") {
                blog.tags = Some(tags.split(',').map(|tag| tag.trim().to_owned()).collect());
            }

            if let Some(cover_image) = header_map.remove("cover_image") {
                blog.cover_image = cover_image.to_owned();
            }

            blog.dob = if let Some(dob) = header_map.remove("dob") {
                dob.to_owned()
            } else {
                format!(
                    "{}",
                    DateTime::<Utc>::from(path.metadata()?.created()?)
                        .date()
                        .format("YYYY/mm/dd")
                )
            }
        }

        Ok(blog)
    }
}

pub struct MarkdownData {
    pub content: String,
    pub file_path: PathBuf,
}

pub fn read_markdown(file_path: &Path) -> Result<String> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(Error::new(e)),
    }
}

impl MarkdownData {
    pub fn from_file(file_path: PathBuf) -> HbpResult<MarkdownData> {
        let content = read_markdown(&file_path)?;

        Ok(MarkdownData { content, file_path })
    }

    pub fn title(&self) -> String {
        if let Some(title) = self.file_path.file_name() {
            title.to_string_lossy().into_owned()
        } else {
            self.file_path.to_string_lossy().into_owned()
        }
    }
}
