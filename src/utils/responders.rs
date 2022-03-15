use httpstatus::StatusCode;
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Header, Status};
use rocket::response::{Responder, Response, Result};
use std::io::Cursor;

#[allow(dead_code)]
pub enum HbpContent {
    Plain(String),
    Html(String),
    Json(String),
    Redirect(String),
    File(Box<std::path::Path>),
}

pub struct HbpResponse {
    pub status_code: StatusCode,
    pub content: HbpContent,
}

#[allow(dead_code)]
impl HbpResponse {
    #[allow(dead_code)]
    pub fn empty() -> HbpResponse {
        HbpResponse {
            status_code: StatusCode::Ok,
            content: HbpContent::Plain(String::new()),
        }
    }
    pub fn text(text: &str, status_code: StatusCode) -> HbpResponse {
        HbpResponse {
            status_code,
            content: HbpContent::Plain(text.to_owned()),
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
        let content = format!("{} | {}", status_code.as_u16(), status_code.reason_phrase());

        HbpResponse {
            status_code,
            content: HbpContent::Plain(content),
        }
    }
    pub fn unauthorized() -> HbpResponse {
        HbpResponse::status(StatusCode::Unauthorized)
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
    #[allow(dead_code)]
    pub fn redirect(uri: rocket::http::uri::Uri) -> HbpResponse {
        let location = match uri.absolute() {
            Some(uri) => uri.path().as_str().to_owned(),
            None => uri.origin().unwrap().path().as_str().to_owned(),
        };

        HbpResponse {
            status_code: StatusCode::MovedPermanently,
            content: HbpContent::Redirect(location),
        }
    }
}

impl<'r> Responder<'r, 'r> for HbpResponse {
    // ! FIXME: Change `respond_to` into async when async Traits roll out...!
    fn respond_to(self, request: &rocket::Request<'_>) -> Result<'r> {
        let mut response_builder = Response::build();

        let status = Status::from_code(self.status_code.as_u16()).unwrap();
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
            HbpContent::Redirect(path) => {
                response_builder
                    .header(ContentType::HTML)
                    .status(Status::MovedPermanently)
                    .header(Header::new("Location", path));
            }
            HbpContent::File(file_path) => {
                return futures::executor::block_on(NamedFile::open(&file_path)).respond_to(request)
            }
        }

        Ok(response_builder.finalize())
    }
}
