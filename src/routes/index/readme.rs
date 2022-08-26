use std::path::PathBuf;

use httpstatus::StatusCode;
use rocket::response::Redirect;
use rocket::{get, routes, Route};

use crate::shared::entities::markdown::Markdown;
use crate::utils::markdown;
use crate::utils::responders::HbpResponse;
use crate::utils::template::IndexLayout;
use crate::utils::types::HbpResult;

#[get("/README.md")]
async fn readme_md() -> HbpResult<HbpResponse> {
    let file_path = PathBuf::from("README.md");
    let markdown_data = Markdown::from_markdown(&file_path)?;

    let html_result = async {
        if markdown::is_marp(&markdown_data.content) {
            markdown::render_marp(&markdown_data).await
        } else {
            markdown::render_markdown(
                &markdown_data,
                IndexLayout::from_title(markdown_data.title.to_owned()),
            )
            .await
        }
    };

    let html = html_result.await?;
    Ok(HbpResponse::html(html, StatusCode::Ok))
}

#[get("/")]
fn index() -> Redirect {
    Redirect::moved("/README.md")
}

pub fn index_routes() -> Vec<Route> {
    routes![index, readme_md]
}
