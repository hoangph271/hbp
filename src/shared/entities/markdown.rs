use crate::shared::interfaces::ApiError;
use crate::utils::responders::HbpResult;
use crate::utils::url_encode_path;
use anyhow::Result;
use chrono::{DateTime, Utc};
use httpstatus::StatusCode;
use mustache::{Data, EncoderError, MapBuilder};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Default)]
pub struct FsoMarkdown {
    pub title: String,
    pub file_name: String,
    pub author: String,
    pub content: String,
    pub dob: String,
    pub tags: Option<Vec<String>>,
    pub cover_image: String,
    pub url: String,
}

impl From<FsoMarkdown> for Data {
    fn from(markdown: FsoMarkdown) -> Data {
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
                        builder = builder
                            .push(tag)
                            .unwrap_or_else(|e| panic!("push tag failed: {e:?}"));
                    }

                    builder
                })
            }

            Ok(map_builder.build())
        };

        insert_fields().unwrap_or_else(|e| panic!("insert_fields fail: {e}"))
    }
}

pub fn extract_markdown_header_content(content: &str) -> Option<String> {
    if let Some(header_comment) = Regex::new("<!--((.|\n)*?)-->").ok()?.find(content) {
        if header_comment.start() != 0 {
            None
        } else {
            let (start, end) = (
                header_comment.start() + "<!--".len(),
                header_comment.end() - "-->".len(),
            );
            let header_content = (content[start..end]).to_string();
            Some(header_content)
        }
    } else {
        None
    }
}

impl FsoMarkdown {
    pub fn from_markdown(path: &Path) -> HbpResult<FsoMarkdown> {
        if !path.exists() {
            let msg = format!("{} NOT exists", path.to_string_lossy());
            return Err(ApiError::from_message(&msg, StatusCode::BadRequest).into());
        }

        let mut markdown = FsoMarkdown {
            // TODO: Abstract this map_err
            content: fs::read_to_string(path)?,
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            url: url_encode_path(&path.to_string_lossy()),
            ..FsoMarkdown::default()
        };

        if let Some(header_comment) = extract_markdown_header_content(&markdown.content) {
            let mut header_map: HashMap<String, String> = header_comment
                .trim()
                .split('\n')
                .map(|line| {
                    let colon_index = line.find(':').unwrap_or_else(|| {
                        panic!("header_comment value lines MUST contain a colon")
                    });

                    (
                        (line[..colon_index]).trim().to_string(),
                        (line[colon_index + 1..]).trim().to_string(),
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

            if let Some(dob) = header_map.remove("dob") {
                markdown.dob = dob;
            }
        }

        if markdown.title.is_empty() {
            markdown.title = match path.file_name() {
                Some(file_name) => file_name.to_string_lossy().to_string(),
                None => "Untitled...!".to_owned(),
            };
        }

        if markdown.dob.is_empty() {
            markdown.dob = format!(
                "{}",
                DateTime::<Utc>::from(path.metadata()?.created()?)
                    .date_naive()
                    .format("%m/%d/%Y")
            );
        }

        Ok(markdown)
    }
}

#[derive(Debug, Serialize)]
pub struct FsoDirectory {
    pub title: String,
    pub url: String,
}

#[derive(Serialize, Debug)]
pub enum FsoFileType {
    Markdown(FsoMarkdown),
    Unknown,
}

#[derive(Serialize, Debug)]
pub struct FsoFile {
    pub title: String,
    pub url: String,
    pub fso_type: FsoFileType,
}

impl FsoFile {
    pub fn markdown(title: String, url: String, fso_markdown: FsoMarkdown) -> FsoFile {
        Self {
            title,
            url,
            fso_type: FsoFileType::Markdown(fso_markdown),
        }
    }

    pub fn unknown(title: String, url: String) -> FsoFile {
        Self {
            title,
            url,
            fso_type: FsoFileType::Unknown,
        }
    }
}
impl From<FsoFile> for FsoEntry {
    fn from(fso_file: FsoFile) -> Self {
        Self::FsoFile(fso_file)
    }
}

#[derive(Serialize, Debug)]
pub enum FsoEntry {
    FsoFile(FsoFile),
    FsoDirectory(FsoDirectory),
}

impl FsoEntry {
    pub fn from_path(path: &PathBuf) -> Self {
        let title = path
            .file_name()
            .map(|filename| filename.to_string_lossy())
            .unwrap_or(path.to_string_lossy())
            .to_string();
        let url = url_encode_path(&path.to_string_lossy());

        if path.is_dir() {
            return Self::FsoDirectory(FsoDirectory { title, url });
        }

        if let Some(file_ext) = path.extension().map(|f| f.to_string_lossy()) {
            return match file_ext.to_lowercase().as_str() {
                "md" => {
                    let fso_markdown = FsoMarkdown::from_markdown(path).unwrap();
                    FsoFile::markdown(title, url, fso_markdown)
                }
                _ => FsoFile::unknown(title, url),
            }.into();
        }

        FsoFile::unknown(title, url).into()
    }
}
