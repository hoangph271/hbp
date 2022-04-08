use crate::shared::entities::markdown::Markdown;
use crate::utils::responders::HbpResponse;
use crate::utils::template::{render_default_layout, DefaultLayoutData};
use chrono::{NaiveDate};
use mustache::MapBuilder;
use std::fs::read_dir;

#[get("/")]
pub fn index() -> HbpResponse {
    let mut markdowns: Vec<Markdown> = read_dir("markdown/blogs")
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();

            Markdown::from_markdown(&entry.path()).unwrap()
        })
        .collect();

    markdowns.sort_by(|m1, m2| {
        const DATE_FORMAT: &str = "%m/%d/%Y";
        NaiveDate::parse_from_str(&m1.dob, DATE_FORMAT)
            .unwrap()
            .cmp(&NaiveDate::parse_from_str(&m2.dob, DATE_FORMAT).unwrap())
    });

    let html = render_default_layout(
        "blogs/index.html",
        Some(DefaultLayoutData::only_title("Blogs")),
        Some(
            MapBuilder::new()
                .insert_vec("blogs", |builder| {
                    let mut builder = builder;

                    for blog in &markdowns {
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
