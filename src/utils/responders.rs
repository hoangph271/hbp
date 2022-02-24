use httpstatus::StatusCode;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Response, Result};
use rocket::tokio::fs::{File};
use std::io::{Cursor};

pub enum HbpContent {
    Plain(String),
    Html(String),
    File(Box<ContentType>, File),
}

pub struct HbpResponse {
    status_code: StatusCode,
    content: HbpContent,
}

impl HbpResponse {
    pub fn ok(content: HbpContent) -> HbpResponse {
        HbpResponse {
            status_code: StatusCode::Ok,
            content,
        }
    }
    pub fn status(status_code: StatusCode) -> HbpResponse {
        HbpResponse::status_text(status_code.clone(), status_code.reason_phrase())
    }
    pub fn status_text(status_code: StatusCode, content: &str) -> HbpResponse {
        HbpResponse {
            status_code,
            content: HbpContent::Plain(content.to_owned()),
        }
    }
}

impl<'r> Responder<'r, 'r> for HbpResponse {
    fn respond_to(self, _: &rocket::Request<'_>) -> Result<'r> {
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
            HbpContent::File(mime, file) => {
                response_builder.header(*mime).streamed_body(file);
            }
        }

        Ok(response_builder.finalize())
    }
}
