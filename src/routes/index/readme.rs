use std::path::PathBuf;

use httpstatus::StatusCode;
use rocket::response::Redirect;
use rocket::{get, routes, Route};

use crate::shared::entities::markdown::FsoMarkdown;
use crate::utils::fso;
use crate::utils::responders::{HbpResponse, HbpResult};
use crate::utils::template::IndexLayout;

#[get("/README.md")]
async fn readme_md() -> HbpResult<HbpResponse> {
    let file_path = PathBuf::from("README.md");
    let markdown_data = FsoMarkdown::from_markdown(&file_path)?;

    let html_result = async {
        if fso::is_marp(&markdown_data.content) {
            fso::render_marp(&markdown_data).await
        } else {
            fso::render_markdown(
                &markdown_data,
                IndexLayout::from_title(&markdown_data.title),
            )
            .await
        }
    };

    let html = html_result.await?;
    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/")]
fn index() -> Redirect {
    Redirect::found("/markdown/AboutMe.md")
}

pub fn index_routes() -> Vec<Route> {
    routes![index, readme_md]
}
