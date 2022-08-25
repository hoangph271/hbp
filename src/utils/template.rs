use crate::shared::entities::markdown::Markdown;
use crate::shared::interfaces::ApiError;
use crate::utils::types::HbpResult;
use httpstatus::StatusCode;
use mustache::Template;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::vec;

use super::auth::AuthPayload;
use super::markdown::markdown_to_html;
use super::string::url_encode_path;

fn compile_template(path: &PathBuf) -> HbpResult<Template> {
    mustache::compile_path(Path::new("template").join(path)).map_err(|e| e.into())
}

#[derive(Serialize)]
pub struct Templater {
    template_path: PathBuf,
}

impl Templater {
    pub fn new(template_path: PathBuf) -> Self {
        Templater { template_path }
    }

    pub fn error_page() -> Self {
        Templater::new("static/error.html".into())
    }

    pub fn to_html(&self, data: impl Serialize) -> HbpResult<String> {
        let template = compile_template(&self.template_path)?;

        let mut bytes = vec![];
        template.render(&mut bytes, &data)?;

        Ok(std::str::from_utf8(&bytes)?.to_owned())
    }

    pub fn to_html_page(
        &self,
        data: impl Serialize + std::fmt::Debug,
        layout_data: IndexLayout,
    ) -> HbpResult<String> {
        #[derive(Serialize)]
        struct RenderData {
            raw_content: String,
            title: String,
            moveup_urls: Vec<MoveUpUrl>,
            username: String,
        }

        Templater::new("index.html".into()).to_html(RenderData {
            raw_content: self.to_html(data)?,
            title: layout_data.title,
            moveup_urls: layout_data.moveup_urls,
            username: layout_data.username,
        })
    }
}

#[derive(Default, Serialize)]
pub struct MoveUpUrl {
    pub title: String,
    pub url: String,
}
impl MoveUpUrl {
    pub fn from_path(file_path: &Path) -> Vec<MoveUpUrl> {
        match file_path.parent() {
            Some(parent_path) => {
                let mut moveup_urls: Vec<MoveUpUrl> = vec![];

                for sub_path in parent_path.iter() {
                    let title = sub_path.to_string_lossy().to_string();
                    let prev_url: String = match moveup_urls.last() {
                        Some(moveup_url) => (*moveup_url).url.clone(),
                        None => "/".to_string(),
                    };

                    let mut url = PathBuf::from(prev_url);
                    url.push(title.clone());

                    moveup_urls.push(MoveUpUrl {
                        title,
                        url: url_encode_path(&url.to_string_lossy()),
                    })
                }

                moveup_urls
            }
            None => vec![],
        }
    }
}

#[derive(Default, Serialize)]
pub struct IndexLayout {
    title: String,
    username: String,
    moveup_urls: Vec<MoveUpUrl>,
    raw_content: String,
}

impl IndexLayout {
    pub fn from_title(title: String) -> Self {
        Self::default().title(title)
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;

        self
    }

    pub fn maybe_auth(mut self, jwt: Option<AuthPayload>) -> Self {
        let username = if let Some(jwt) = jwt {
            match jwt {
                AuthPayload::User(user) => user.sub,
                AuthPayload::UserResource(user_resouce) => user_resouce.sub,
            }
        } else {
            "".to_owned()
        };

        self.username = username;

        self
    }

    pub fn username(mut self, username: &str) -> Self {
        self.username = username.to_owned();

        self
    }

    pub fn moveup_urls(mut self, moveup_urls: Vec<MoveUpUrl>) -> Self {
        self.moveup_urls = moveup_urls;

        self
    }
}

#[derive(Serialize, Debug)]
pub struct MarkdownTemplate {
    markdown_html: String,
    markdown_signed_url: String,
    markdown_title: String,
}

impl MarkdownTemplate {
    pub fn of(markdown: &Markdown, signed_url: Option<String>) -> MarkdownTemplate {
        MarkdownTemplate {
            markdown_html: markdown_to_html(&markdown.content),
            markdown_title: markdown.title.clone(),
            markdown_signed_url: signed_url.unwrap_or_default(),
        }
    }
}

impl From<std::str::Utf8Error> for ApiError {
    fn from(e: std::str::Utf8Error) -> Self {
        ApiError::from_message(
            &format!("UTF8 Issue: , {e}"),
            StatusCode::InternalServerError,
        )
    }
}
impl From<mustache::Error> for ApiError {
    fn from(e: mustache::Error) -> Self {
        ApiError {
            with_ui: false,
            errors: vec![e.to_string()],
            status_code: match e {
                mustache::Error::InvalidStr => StatusCode::UnprocessableEntity,
                mustache::Error::NoFilename => StatusCode::NotFound,
                _ => StatusCode::InternalServerError,
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorPage {
    pub error_text: String,
    pub action_html: String,
}

impl ErrorPage {
    pub fn from_status(status_code: &StatusCode) -> Self {
        Self {
            error_text: status_text(status_code),
            action_html: action_html_from(status_code),
        }
    }

    pub fn action_html(mut self, action_html: String) -> Self {
        self.action_html = action_html;
        self
    }
}

fn action_html_from(status_code: &StatusCode) -> String {
    match status_code {
        StatusCode::Unauthorized => action_html_for_401(None),
        _ => r#"
            <p>
                Click <a href="/">here</a> to get home...!
            </p>"#
            .to_owned(),
    }
}

pub fn action_html_for_401(redirect_url: Option<String>) -> String {
    let href = if let Some(redirect_url) = redirect_url {
        format!("/users/login?redirect_url={redirect_url}")
    } else {
        "/users/login".to_owned()
    };

    format!(
        "
        <p>
            Click <a href=\"{href}\">here</a> to signin...!
        </p>"
    )
}

pub fn status_text(status_code: &StatusCode) -> String {
    format!("{} | {}", status_code.as_u16(), status_code.reason_phrase())
}
