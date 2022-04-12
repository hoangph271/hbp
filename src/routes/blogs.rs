use crate::shared::entities::markdown::Markdown;
use crate::utils::auth::AuthPayload;
use crate::utils::markdown::{markdown_from_dir, render_markdown_list};
use crate::utils::responders::HbpResponse;
use crate::utils::template::DefaultLayoutData;
use chrono::NaiveDate;

#[get("/")]
pub fn index(jwt: Option<AuthPayload>) -> HbpResponse {
    let mut markdowns: Vec<Markdown> = markdown_from_dir(&"markdown/blogs");

    markdowns.sort_by(|m1, m2| {
        const DATE_FORMAT: &str = "%m/%d/%Y";
        NaiveDate::parse_from_str(&m2.dob, DATE_FORMAT)
            .unwrap()
            .cmp(&NaiveDate::parse_from_str(&m1.dob, DATE_FORMAT).unwrap())
    });

    HbpResponse::html(
        &render_markdown_list(
            DefaultLayoutData::only_title("Blogs").maybe_auth(jwt),
            markdowns,
        ),
        None,
    )
}
