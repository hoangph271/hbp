use futures::Future;
use hbp_types::{ApiItem, ApiList};
use httpstatus::StatusCode;
use rocket::http::ContentType;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::shared::interfaces::{ApiError, ApiResult};

use super::template::{action_html_for_401, status_text, ErrorPage, IndexLayout, Templater};

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

#[derive(Debug)]
pub struct HbpError {
    pub api_error: ApiError,
}

#[derive(Debug)]
pub enum HbpJson<T: Serialize> {
    Item(ApiItem<T>),
    List(ApiList<T>),
    Empty,
}

pub type HbpApiResult<T: Serialize> = Result<HbpJson<T>, HbpError>;

pub type HbpResult<T> = Result<T, HbpError>;

mod hbp_response_impls {
    use super::{json_stringify, HbpContent, HbpError, HbpJson, HbpResponse};
    use crate::{data::lib::OrmError, utils::status_from};
    use hbp_types::{ApiError, ApiItem, ApiList};
    use httpstatus::StatusCode;
    use image::ImageError;
    use log::error;
    use okapi::openapi3::Responses;
    use rocket::{
        fs::NamedFile,
        http::{ContentType, Header, Status},
        response::Responder,
        Response,
    };
    use rocket_okapi::gen::OpenApiGenerator;
    use rocket_okapi::response::OpenApiResponderInner;
    use serde::Serialize;
    use std::{error::Error, io::Cursor};

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

    impl<'r> Responder<'r, 'r> for HbpError {
        fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
            let res = HbpResponse {
                status_code: self.api_error.status_code,
                content: HbpContent::Json(json_stringify(&self.api_error)),
            };

            res.respond_to(request)
        }
    }

    impl<'r, T> Responder<'r, 'r> for HbpJson<T>
    where
        T: Serialize,
    {
        fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
            let (status_code, content) = match self {
                HbpJson::Item(item) => (item.status_code, json_stringify(&item)),
                HbpJson::List(list) => (list.status_code, json_stringify(&list)),
                HbpJson::Empty => (StatusCode::Ok, "".to_owned()),
            };

            let res = HbpResponse {
                status_code,
                content: HbpContent::Json(content),
            };

            res.respond_to(request)
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

    impl OpenApiResponderInner for HbpResponse {
        fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }

    impl OpenApiResponderInner for HbpError {
        fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }

    impl<T: Serialize> OpenApiResponderInner for HbpJson<T> {
        fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }

    impl From<ApiError> for HbpResponse {
        fn from(e: ApiError) -> Self {
            HbpResponse {
                status_code: e.status_code,
                content: HbpContent::Json(json_stringify(&e)),
            }
        }
    }

    impl From<ApiError> for HbpError {
        fn from(api_error: ApiError) -> Self {
            Self { api_error }
        }
    }

    impl From<ImageError> for HbpError {
        fn from(e: ImageError) -> Self {
            error!("ImageError: {e}");

            match e {
                ImageError::Decoding(e) => {
                    ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
                }
                ImageError::Encoding(e) => {
                    ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
                }
                ImageError::Parameter(e) => {
                    ApiError::from_message(&e.to_string(), StatusCode::BadRequest)
                }
                ImageError::Limits(e) => {
                    ApiError::from_message(&e.to_string(), StatusCode::UnprocessableEntity)
                }
                ImageError::Unsupported(e) => {
                    ApiError::from_message(&e.to_string(), StatusCode::UnprocessableEntity)
                }
                ImageError::IoError(e) => {
                    ApiError::from_message(&format!("{e}"), StatusCode::InternalServerError)
                }
            }
            .into()
        }
    }

    impl From<std::str::Utf8Error> for HbpError {
        fn from(e: std::str::Utf8Error) -> Self {
            ApiError::from_message(
                &format!("UTF8 Issue: , {e}"),
                StatusCode::InternalServerError,
            )
            .into()
        }
    }

    impl From<mustache::Error> for HbpError {
        fn from(e: mustache::Error) -> Self {
            let status_code = match e {
                mustache::Error::InvalidStr => StatusCode::UnprocessableEntity,
                mustache::Error::NoFilename => StatusCode::NotFound,
                _ => StatusCode::InternalServerError,
            };

            ApiError::new(status_code, vec![e.to_string()]).into()
        }
    }

    impl From<reqwest::Error> for HbpError {
        fn from(e: reqwest::Error) -> Self {
            error!("[reqwest::Error]: {e}");

            let msg = match e.source() {
                Some(source) => format!("{:?}", source),
                None => "Unknown error".to_owned(),
            };

            ApiError::from_message(
                &msg,
                if let Some(status_code) = e.status() {
                    status_code.as_u16().into()
                } else {
                    StatusCode::InternalServerError
                },
            )
            .into()
        }
    }

    impl From<std::io::Error> for HbpError {
        fn from(e: std::io::Error) -> Self {
            ApiError {
                with_ui: false,
                status_code: StatusCode::InternalServerError,
                errors: vec![format!("{e}")],
            }
            .into()
        }
    }

    impl<T: Serialize> From<ApiItem<T>> for HbpJson<T> {
        fn from(item: ApiItem<T>) -> Self {
            HbpJson::Item(item)
        }
    }

    impl<T: Serialize> From<ApiList<T>> for HbpJson<T> {
        fn from(list: ApiList<T>) -> Self {
            HbpJson::List(list)
        }
    }

    impl From<OrmError> for HbpError {
        fn from(e: OrmError) -> Self {
            match e {
                OrmError::NotFound => ApiError::not_found().into(),
            }
            // match e {
            //     Ok(post) => HbpResponse::json(post, None),
            //     Err(e) => match e {
            //         OrmError::NotFound => Err(ApiError::from_status(StatusCode::NotFound)),
            //     },
            // }
        }
    }
}

fn json_stringify(json: &impl Serialize) -> String {
    serde_json::to_string(&json).unwrap_or_else(|e| panic!("json_stringify failed: {e}"))
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
