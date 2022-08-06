use crate::utils::types::HbpResult;
use mustache::Template;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::vec;

use super::auth::AuthPayload;
use super::string::url_encode_path;

fn compile_template(path: &PathBuf) -> HbpResult<Template> {
    mustache::compile_path(Path::new("template").join(path)).map_err(|e| e.into())
}

#[derive(Serialize)]
pub struct TemplateRenderer {
    template_path: PathBuf,
}

impl TemplateRenderer {
    pub fn new(template_path: PathBuf) -> Self {
        TemplateRenderer { template_path }
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
        layout_data: IndexLayoutData,
    ) -> HbpResult<String> {
        #[derive(Serialize)]
        struct RenderData {
            raw_content: String,
            title: String,
            moveup_urls: Vec<MoveUpUrl>,
            username: String,
        }

        TemplateRenderer::new("index.html".into()).to_html(RenderData {
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

                for sub_path in parent_path.iter().chain(file_path.file_name().into_iter()) {
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
pub struct IndexLayoutData {
    title: String,
    username: String,
    moveup_urls: Vec<MoveUpUrl>,
    raw_content: String,
}
impl IndexLayoutData {
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();

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
    pub fn only_title(title: &str) -> Self {
        Self::default().title(title)
    }
    pub fn moveup_urls(mut self, moveup_urls: Vec<MoveUpUrl>) -> Self {
        self.moveup_urls = moveup_urls;

        self
    }
}
