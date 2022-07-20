use rocket::get;

use crate::shared::entities::markdown::*;
use crate::utils::auth::AuthPayload;
use crate::utils::markdown::{markdown_from_dir, render_markdown_list};
use crate::utils::responders::HbpResponse;
use crate::utils::template::DefaultLayoutData;

#[get("/")]
pub fn index(jwt: Option<AuthPayload>) -> HbpResponse {
    let markdowns: Vec<MarkdownOrMarkdownDir> = markdown_from_dir(&"markdown/blogs").unwrap();

    // FIXME: Now with dir, how to sort...?
    // markdowns.sort_by(|m1, m2| {
    //     const DATE_FORMAT: &str = "%m/%d/%Y";
    //     NaiveDate::parse_from_str(&m2.dob, DATE_FORMAT)
    //         .unwrap()
    //         .cmp(&NaiveDate::parse_from_str(&m1.dob, DATE_FORMAT).unwrap())
    // });

    HbpResponse::html(
        &render_markdown_list(
            DefaultLayoutData::only_title("Blogs").maybe_auth(jwt),
            markdowns,
        ),
        None,
    )
}
