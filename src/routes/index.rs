use std::path::PathBuf;

use httpstatus::StatusCode;
use rocket::http::Status;
use rocket::response::Redirect;

use crate::shared::entities::markdown::Markdown;
use crate::utils::markdown;
use crate::utils::responders::HbpResponse;
use crate::utils::template::DefaultLayoutData;

#[get("/README.md")]
pub async fn readme_md() -> HbpResponse {
    let file_path = PathBuf::from("README.md");

    match Markdown::from_markdown(&file_path) {
        Ok(markdown_data) => {
            let html_result = async {
                if markdown::is_marp(&markdown_data.content) {
                    markdown::render_marp(&markdown_data, None).await
                } else {
                    let html = markdown::render_markdown(
                        &markdown_data,
                        Some(DefaultLayoutData::only_title(&markdown_data.title)),
                    )
                    .await;
                    html
                }
            };

            match html_result.await {
                Ok(html) => HbpResponse::html(&html, None),
                Err(e) => {
                    error!("{}", e);
                    HbpResponse::status(StatusCode::InternalServerError)
                }
            }
        }
        Err(e) => {
            error!("{e}");
            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

#[get("/")]
pub fn index() -> Redirect {
    Redirect::moved("/README.md")
}

#[get("/")]
pub fn get_dev_null() -> Status {
    Status::Ok
}
#[post("/")]
pub fn post_dev_null() -> Status {
    Status::Ok
}
#[put("/")]
pub fn put_dev_null() -> Status {
    Status::Ok
}
#[delete("/")]
pub fn delete_dev_null() -> Status {
    Status::Ok
}
