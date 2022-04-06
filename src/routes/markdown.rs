use crate::utils::{
    auth::{AuthPayload, UserPayload},
    markdown,
    markdown::MarkdownData,
    responders::{HbpContent, HbpResponse},
    types::MarkdownMetadata,
};
use httpstatus::StatusCode;
use mustache::Data;
use std::path::PathBuf;

#[get("/<sub_path..>")]
pub async fn markdown_file(sub_path: PathBuf) -> HbpResponse {
    let file_path = PathBuf::from("markdown").join(sub_path.clone());

    if !markdown::is_markdown(&sub_path) {
        return HbpResponse::file(file_path);
    }

    match MarkdownData::from_file(file_path.clone()) {
        Ok(markdown_data) => {
            match markdown::render_markdown(
                &markdown_data,
                Some(vec![(
                    "title".to_owned(),
                    Data::String(file_path.to_string_lossy().into_owned()),
                )]),
            )
            .await
            {
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

    let title = match file_path.file_name() {
        Some(file_name) => file_name.to_string_lossy(),
        None => file_path_str,
    };

    let ogp_metadata = MarkdownMetadata::of_markdown(&file_path);

    let extra_data = vec![
        ("title".to_owned(), Data::String(title.into_owned())),
        (
            "ogp_metadata".to_owned(),
            match ogp_metadata {
                Some(ogp_metadata) => ogp_metadata.to_mustache_data(),
                None => Data::Null,
            },
        ),
    ];

    if !file_path.exists() {
        info!("{:?} not exists", file_path.to_string_lossy());
        return HbpResponse::not_found();
    }

    if !markdown::is_markdown(&file_path) {
        return HbpResponse::file(file_path);
    }

    if let Ok(markdown_data) = MarkdownData::from_file(file_path) {
        return match markdown::render_markdown(&markdown_data, Some(extra_data)).await {
            Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
            Err(e) => {
                error!("{}", e);
                HbpResponse::internal_server_error()
            }
        };
    }

    HbpResponse::internal_server_error()
}
