use log::error;
use rocket::get;

use crate::shared::entities::markdown::*;
use crate::utils::auth::AuthPayload;
use crate::utils::markdown::{markdown_from_dir, render_markdown_list};
use crate::utils::responders::HbpResponse;
use crate::utils::template::IndexLayoutData;

#[get("/")]
pub fn index(jwt: Option<AuthPayload>) -> HbpResponse {
    let markdowns: Vec<MarkdownOrMarkdownDir> = match markdown_from_dir(&"markdown/blogs") {
        Ok(markdowns) => markdowns,
        Err(e) => {
            error!("markdown_from_dir failed: {:?}", e);

            return e.into();
        }
    };

    // FIXME: Now with dir, how to sort...?
    // markdowns.sort_by(|m1, m2| {
    //     const DATE_FORMAT: &str = "%m/%d/%Y";
    //     NaiveDate::parse_from_str(&m2.dob, DATE_FORMAT)
    //         .unwrap()
    //         .cmp(&NaiveDate::parse_from_str(&m1.dob, DATE_FORMAT).unwrap())
    // });

    match render_markdown_list(
        IndexLayoutData::default().title("Blogs").maybe_auth(jwt),
        markdowns,
    ) {
        Ok(html) => HbpResponse::html(&html, None),
        Err(e) => e.into(),
    }
}
