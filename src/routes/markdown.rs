use crate::shared::entities::markdown::Markdown;
use crate::utils::{
    auth::{AuthPayload, UserPayload},
    markdown,
    responders::{HbpContent, HbpResponse},
};
use httpstatus::StatusCode;
use mustache::Data;
use std::path::{Path, PathBuf};

#[get("/<sub_path..>")]
pub async fn markdown_file(sub_path: PathBuf) -> HbpResponse {
    let file_path = PathBuf::from("markdown").join(sub_path.clone());

    if !markdown::is_markdown(&sub_path) {
        return HbpResponse::file(file_path);
    }

    match Markdown::from_markdown(&file_path) {
        Ok(markdown_data) => {
            match markdown::render_markdown(&markdown_data, markdown_extra_data(&file_path)).await {
                Ok(html) => HbpResponse::html(&html, None),
                Err(_) => HbpResponse::internal_server_error(),
            }
        }
        Err(e) => {
            error!("{e}");

            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}

#[get("/users/<username>/<sub_path..>")]
pub async fn user_markdown_file(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
) -> HbpResponse {
    let file_path = PathBuf::from("markdown")
        .join("users")
        .join(username)
        .join(sub_path.clone());

    let file_path_str = file_path.to_string_lossy();
    let user_assert = |payload: &UserPayload, path: &str| {
        let prefix = PathBuf::from("markdown")
            .join("users")
            .join(payload.sub.clone())
            .to_string_lossy()
            .into_owned();

        path.starts_with(&*prefix)
    };

    if !jwt.match_path(&file_path_str, Some(user_assert)) {
        return HbpResponse::status(StatusCode::Forbidden);
    }

    if !file_path.exists() {
        info!("{:?} not exists", file_path.to_string_lossy());
        return HbpResponse::not_found();
    }

    if !markdown::is_markdown(&file_path) {
        return HbpResponse::file(file_path);
    }

    if let Ok(markdown_data) = Markdown::from_markdown(&file_path) {
        return match markdown::render_markdown(&markdown_data, markdown_extra_data(&file_path))
            .await
        {
            Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
            Err(e) => {
                error!("{}", e);
                HbpResponse::internal_server_error()
            }
        };
    }

    HbpResponse::internal_server_error()
}

fn markdown_extra_data(file_path: &Path) -> Option<Vec<(String, Data)>> {
    if let Ok(markdown) = Markdown::from_markdown(file_path) {
        Some(vec![
            ("title".to_owned(), Data::String(markdown.title.clone())),
            ("og_title".to_owned(), Data::String(markdown.title)),
            ("og_image".to_owned(), Data::String(markdown.cover_image)),
        ])
    } else {
        None
    }
}
