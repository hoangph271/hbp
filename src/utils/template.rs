use crate::shared::entities::markdown::FsoMarkdown;
use httpstatus::StatusCode;
use mustache::Template;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::vec;

use super::auth::AuthPayload;
use super::fso::markdown_to_html;
use super::responders::HbpResult;
use super::url_encode_path;

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

    pub fn index() -> Self {
        Templater::new("index.html".into())
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

#[derive(Default, Serialize, Debug)]
pub struct MoveUpUrl {
    pub title: String,
    pub url: String,
}
impl MoveUpUrl {
    pub fn from_path(file_path: &Path) -> Vec<MoveUpUrl> {
        // TODO: This from_path is too buggy
        match file_path.parent() {
            Some(parent_path) => {
                let mut moveup_urls: Vec<MoveUpUrl> = vec![];

                for sub_path in parent_path.iter() {
                    let title = sub_path.to_string_lossy().to_string();
                    let prev_url: String = match moveup_urls.last() {
                        Some(moveup_url) => moveup_url.url.clone(), // BUG HEERE
                        None => "".to_string(),
                    };

                    let url = format!("{prev_url}/{}", url_encode_path(&title));

                    moveup_urls.push(MoveUpUrl { title, url })
                }

                moveup_urls
            }
            None => vec![],
        }
    }
}

mod test_move_up_url {
    #[cfg(target_family="unix")]
    #[test]
    fn unix_test_move_up_urls_from_long_path() {
        use crate::utils::template::MoveUpUrl;
        use std::path::Path;

        let unix_path = Path::new("markdown/users/hbp/_journal/2023/2023.04");

        let to_url_string = |it: &MoveUpUrl| it.url.to_string();

        let move_up_urls = vec![
            "/markdown",
            "/markdown/users",
            "/markdown/users/hbp",
            "/markdown/users/hbp/_journal",
            "/markdown/users/hbp/_journal/2023",
        ];

        let unix_move_up_urls: Vec<String> = MoveUpUrl::from_path(unix_path)
            .iter()
            .map(to_url_string)
            .collect();

        assert_eq!(unix_move_up_urls, move_up_urls);
    }

    #[cfg(target_family="windows")]
    #[test]
    fn windows_test_move_up_urls_from_long_path() {
        use crate::utils::template::MoveUpUrl;
        use std::path::Path;

        let windows_path = Path::new("markdown\\users\\hbp\\_journal\\2023\\2023.04");

        let to_url_string = |it: &MoveUpUrl| it.url.to_string();

        let move_up_urls = vec![
            "/markdown",
            "/markdown/users",
            "/markdown/users/hbp",
            "/markdown/users/hbp/_journal",
            "/markdown/users/hbp/_journal/2023",
        ];

        let windows_move_up_urls: Vec<String> = MoveUpUrl::from_path(windows_path)
            .iter()
            .map(to_url_string)
            .collect();

        assert_eq!(windows_move_up_urls, move_up_urls);
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
    pub fn from_title(title: &str) -> Self {
        Self::default().title(title)
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();

        self
    }

    pub fn raw_content(mut self, raw_content: &str) -> Self {
        self.raw_content = raw_content.to_string();

        self
    }

    pub fn set_auth(mut self, jwt: Option<AuthPayload>) -> Self {
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
    markdown_url: String,
    signed_url: String,
    markdown_title: String,
}

impl MarkdownTemplate {
    pub fn of(markdown: &FsoMarkdown, signed_url: Option<String>) -> MarkdownTemplate {
        MarkdownTemplate {
            markdown_html: markdown_to_html(&markdown.content),
            markdown_url: markdown.url.clone(),
            markdown_title: markdown.title.clone(),
            signed_url: signed_url.unwrap_or_default(),
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
