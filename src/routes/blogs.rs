use crate::shared::entities::blog::*;
use crate::utils::responders::HbpResponse;
use crate::utils::template::{render_default_layout, DefaultLayoutData};
use chrono::{DateTime, Utc};
use mustache::MapBuilder;
use std::fs::{read, read_dir};

#[get("/")]
pub fn index() -> HbpResponse {
    let blogs: Vec<Blog> = read_dir("markdown/blogs")
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let title = entry.file_name().to_string_lossy().to_string();
            let dob = entry.metadata().unwrap().created().unwrap();
            let dob = format!("{}", DateTime::<Utc>::from(dob).date().format("YYYY/mm/dd"));

            let bytes = read(entry.path()).unwrap();
            let content = String::from_utf8_lossy(&bytes).to_string();

            Blog::from_markdown(&entry.path()).unwrap();

            Blog {
                title,
                content,
                dob,
                tags: None,
            }
        })
        .collect();

    let html = render_default_layout(
        "blogs/index.html",
        Some(DefaultLayoutData::only_title("Blogs")),
        Some(
            MapBuilder::new()
                .insert_vec("blogs", |builder| {
                    let mut builder = builder;

                    for blog in &blogs {
                        builder = builder.push(&blog).unwrap();
                    }

                    builder
                })
                .build(),
        ),
    )
    .unwrap();

    HbpResponse::html(&html, None)
}
