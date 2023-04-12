use httpstatus::StatusCode;
use log::error;
use rocket::get;

use crate::shared::entities::markdown::*;
use crate::utils::auth::AuthPayload;
use crate::utils::fso::{from_dir, render_fso_list};
use crate::utils::responders::HbpResponse;
use crate::utils::template::IndexLayout;

#[get("/")]
pub fn index(jwt: Option<AuthPayload>) -> HbpResponse {
    let markdowns: Vec<FsoEntry> = match from_dir(&"markdown/blogs") {
        Ok(markdowns) => markdowns,
        Err(e) => {
            error!("markdown_from_dir failed: {:?}", e);

            return HbpResponse::from_status(e.api_error.status_code);
        }
    };

    // FIXME: Now with dir, how to sort...?

    match render_fso_list(IndexLayout::from_title("Blogs").set_auth(jwt), markdowns) {
        Ok(html) => HbpResponse::html(html, StatusCode::Ok),
        Err(e) => HbpResponse::from_status(e.api_error.status_code),
    }
}
