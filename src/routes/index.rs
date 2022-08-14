use std::path::PathBuf;

use httpstatus::StatusCode;
use log::*;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{delete, get, post, put, routes, Route};

use crate::shared::entities::markdown::Markdown;
use crate::utils::markdown;
use crate::utils::responders::HbpResponse;
use crate::utils::template::IndexLayoutData;

#[get("/README.md")]
async fn readme_md() -> HbpResponse {
    let file_path = PathBuf::from("README.md");

    match Markdown::from_markdown(&file_path) {
        Ok(markdown_data) => {
            let html_result = async {
                if markdown::is_marp(&markdown_data.content) {
                    markdown::render_marp(&markdown_data).await
                } else {
                    markdown::render_markdown(
                        &markdown_data,
                        IndexLayoutData::default().title(&markdown_data.title),
                    )
                    .await
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
fn index() -> Redirect {
    Redirect::moved("/README.md")
}

#[get("/")]
fn get_dev_null() -> Status {
    Status::Ok
}
#[post("/")]
fn post_dev_null() -> Status {
    Status::Ok
}
#[put("/")]
fn put_dev_null() -> Status {
    Status::Ok
}
#[delete("/")]
fn delete_dev_null() -> Status {
    Status::Ok
}

pub fn dev_null_routes() -> Vec<Route> {
    routes![get_dev_null, post_dev_null, put_dev_null, delete_dev_null]
}

pub fn base_routes() -> Vec<Route> {
    routes![index, readme_md]
}
