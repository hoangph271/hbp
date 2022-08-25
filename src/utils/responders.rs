use futures::Future;
use httpstatus::StatusCode;
use okapi::openapi3::Responses;
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Header, Status};
use rocket::response::{Responder, Response};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::shared::interfaces::{ApiError, ApiResult};

use super::status_from;
use super::template::{action_html_for_401, status_text, ErrorPage, IndexLayout, Templater};
use super::types::HbpResult;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub enum HbpContent {
    Plain(String),
    Html(String),
    Json(String),
    Found(String),
    File(Box<PathBuf>),
    #[serde(skip_serializing, skip_deserializing)]
    Bytes(Vec<u8>, Box<Option<ContentType>>),
    #[serde(skip_serializing, skip_deserializing)]
    NamedTempFile(Box<NamedTempFile>),
}

pub struct HbpResponse {
    pub status_code: StatusCode,
    pub content: HbpContent,
}

impl OpenApiResponderInner for HbpResponse {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            ..Default::default()
        })
    }
}

impl HbpResponse {
    pub fn html(html: String, status_code: StatusCode) -> HbpResponse {
        HbpResponse {
            status_code,
            content: HbpContent::Html(html),
        }
    }

    pub fn ok(content: Option<HbpContent>) -> HbpResponse {
        HbpResponse {
            status_code: StatusCode::Ok,
            content: match content {
                Some(content) => content,
                None => HbpContent::Plain(String::new()),
            },
        }
    }

    pub fn from_status(status_code: StatusCode) -> Self {
        Self {
            content: HbpContent::Plain(status_code.reason_phrase().to_owned()),
            status_code,
        }
    }

    pub fn from_error_status(status_code: StatusCode) -> HbpResponse {
        let render_data = ErrorPage::from_status(&status_code);
        let layout_data = IndexLayout::default().title(status_text(&status_code));

        Templater::error_page()
            .to_html_page(render_data, layout_data)
            .map(|html| HbpResponse::html(html, status_code))
            .unwrap_or_else(HbpResponse::from)
    }

    pub fn unauthorized(redirect_url: Option<String>) -> HbpResponse {
        let status_code = StatusCode::Unauthorized;

        let render_data =
            ErrorPage::from_status(&status_code).action_html(action_html_for_401(redirect_url));
        let layout_data = IndexLayout::from_title(status_text(&status_code));

        Templater::error_page()
            .to_html_page(render_data, layout_data)
            .map(|html| HbpResponse::html(html, status_code))
            .unwrap_or_else(HbpResponse::from)
    }

    pub fn forbidden() -> HbpResponse {
        HbpResponse::from_error_status(StatusCode::Forbidden)
    }

    pub fn json<T: Serialize>(
        content: T,
        status_code: Option<StatusCode>,
    ) -> ApiResult<HbpResponse> {
        let status_code = status_code.unwrap_or(StatusCode::Ok);
        let json = serde_json::to_string(&content)
            .map_err(|e| ApiError::internal_server_error().append_error(e.to_string()))?;

        Ok(HbpResponse {
            status_code,
            content: HbpContent::Json(json),
        })
    }

    pub fn internal_server_error() -> HbpResponse {
        HbpResponse::from_error_status(StatusCode::InternalServerError)
    }

    pub fn not_found() -> HbpResponse {
        HbpResponse::from_error_status(StatusCode::NotFound)
    }

    pub fn redirect(uri: rocket::http::uri::Origin) -> HbpResponse {
        HbpResponse {
            status_code: StatusCode::MovedPermanently,
            content: HbpContent::Found(uri.into_normalized().to_string()),
        }
    }

    pub fn file(path: PathBuf) -> HbpResponse {
        HbpResponse::ok(Some(HbpContent::File(Box::new(path))))
    }

    pub fn temp_file(temp_file: NamedTempFile) -> HbpResponse {
        HbpResponse::ok(Some(HbpContent::NamedTempFile(Box::new(temp_file))))
    }
}

impl<'r> Responder<'r, 'r> for HbpResponse {
    // ! FIXME: Change `respond_to` into async when async Traits roll out...!
    fn respond_to(self, request: &rocket::Request<'_>) -> rocket::response::Result<'r> {
        let mut builder = Response::build();

        let status = status_from(self.status_code);
        builder.status(status);

        match self.content {
            HbpContent::Plain(text) => {
                builder
                    .header(ContentType::Plain)
                    .sized_body(text.len(), Cursor::new(text));
            }
            HbpContent::Html(html) => {
                builder
                    .header(ContentType::HTML)
                    .sized_body(html.len(), Cursor::new(html));
            }
            HbpContent::Json(json) => {
                builder
                    .header(ContentType::JSON)
                    .sized_body(json.len(), Cursor::new(json));
            }
            HbpContent::Found(path) => {
                builder
                    .header(ContentType::HTML)
                    .status(Status::Found)
                    .header(Header::new("Location", path));
            }
            HbpContent::File(file_path) => {
                return futures::executor::block_on(NamedFile::open(&*file_path))
                    .respond_to(request)
            }
            HbpContent::NamedTempFile(tempfile) => {
                return futures::executor::block_on(NamedFile::open(tempfile.into_temp_path()))
                    .respond_to(request)
            }
            HbpContent::Bytes(body, content_type) => {
                builder
                    .header(content_type.unwrap_or(ContentType::Binary))
                    .sized_body(body.len(), Cursor::new(body));
            }
        }

        Ok(builder.finalize())
    }
}

impl From<HbpResponse> for Response<'_> {
    fn from(hbp_response: HbpResponse) -> Response<'static> {
        let mut response_builder = Response::build();

        let status = status_from(hbp_response.status_code);
        response_builder.status(status);

        response_builder.finalize()
    }
}

pub async fn wrap_api_handler<R, T>(handler: impl FnOnce() -> R) -> HbpResult<T>
where
    R: Future<Output = HbpResult<T>>,
{
    match handler().await {
        Ok(val) => Ok(val),
        Err(e) => {
            log::error!("{e:?}");
            Err(e)
        }
    }
}
