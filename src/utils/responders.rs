use crate::utils::template::IndexLayoutData;
use futures::Future;
use httpstatus::StatusCode;
use okapi::openapi3::Responses;
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Header, Status};
use rocket::response::{Responder, Response, Result as ResResult};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;

use super::status_from;
use super::template::TemplateRenderer;
use super::types::{HbpError, HbpResult};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, JsonSchema)]
pub enum HbpContent {
    Plain(String),
    Html(String),
    Json(String),
    Found(String),
    File(Box<PathBuf>),
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
    pub fn html(html: &str, status_code: Option<StatusCode>) -> HbpResponse {
        HbpResponse {
            status_code: status_code.unwrap_or(StatusCode::Ok),
            content: HbpContent::Html(html.to_owned()),
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

    pub fn status(status_code: StatusCode) -> HbpResponse {
        let status_code = StatusCode::from(status_code.as_u16());

        #[derive(Serialize, Debug)]
        struct RenderData {
            error_text: String,
            action_html: String,
        }

        let html = TemplateRenderer::new("static/error.html".into())
            .to_html_page(
                RenderData {
                    error_text: format!(
                        "{} | {}",
                        status_code.as_u16(),
                        status_code.reason_phrase()
                    ),
                    action_html: action_html_for(&status_code),
                },
                IndexLayoutData::default().title(status_code.reason_phrase()),
            )
            .unwrap();

        HbpResponse {
            status_code,
            content: HbpContent::Html(html),
        }
    }

    pub fn forbidden() -> HbpResponse {
        HbpResponse::status(StatusCode::Forbidden)
    }

    pub fn json<T: serde::Serialize>(content: T, status_code: Option<StatusCode>) -> HbpResponse {
        let json = serde_json::to_string(&content).expect("Stringify JSON failed");

        let status_code = match status_code {
            Some(status_code) => status_code,
            None => httpstatus::StatusCode::Ok,
        };

        HbpResponse {
            status_code,
            content: HbpContent::Json(json),
        }
    }

    pub fn internal_server_error() -> HbpResponse {
        HbpResponse::status(StatusCode::InternalServerError)
    }

    pub fn not_found() -> HbpResponse {
        HbpResponse::status(StatusCode::NotFound)
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
}

impl<'r> Responder<'r, 'r> for HbpResponse {
    // ! FIXME: Change `respond_to` into async when async Traits roll out...!
    fn respond_to(self, request: &rocket::Request<'_>) -> ResResult<'r> {
        let mut response_builder = Response::build();

        let status = status_from(self.status_code);
        response_builder.status(status);

        match self.content {
            HbpContent::Plain(text) => {
                response_builder
                    .header(ContentType::Plain)
                    .sized_body(text.len(), Cursor::new(text));
            }
            HbpContent::Html(html) => {
                response_builder
                    .header(ContentType::HTML)
                    .sized_body(html.len(), Cursor::new(html));
            }
            HbpContent::Json(json) => {
                response_builder
                    .header(ContentType::JSON)
                    .sized_body(json.len(), Cursor::new(json));
            }
            HbpContent::Found(path) => {
                response_builder
                    .header(ContentType::HTML)
                    .status(Status::Found)
                    .header(Header::new("Location", path));
            }
            HbpContent::File(file_path) => {
                return futures::executor::block_on(NamedFile::open(&*file_path))
                    .respond_to(request)
            }
        }

        Ok(response_builder.finalize())
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

impl From<HbpError> for HbpResponse {
    fn from(e: HbpError) -> Self {
        HbpResponse {
            status_code: e.status_code,
            content: HbpContent::Plain(e.msg),
        }
    }
}

fn action_html_for(status_code: &StatusCode) -> String {
    match status_code {
        StatusCode::Unauthorized => {
            r#"
        <p>
            Click <a href="/users/login">here</a> to signin...!
        </p>"#
        }
        _ => {
            r#"
        <p>
            Click <a href="/">here</a> to get home...!
        </p>"#
        }
    }
    .to_owned()
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
