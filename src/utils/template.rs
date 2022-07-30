use crate::utils::types::HbpResult;
use mustache::Template;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::vec;

use super::auth::AuthPayload;

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
        data: impl Serialize,
        layout_data: IndexLayoutData,
    ) -> HbpResult<String> {
        #[derive(Serialize)]
        struct RenderData {
            raw_content: String,
            title: String,
            moveup_url: String,
            username: String,
        }

        TemplateRenderer::new("index.html".into()).to_html(RenderData {
            raw_content: self.to_html(data)?,
            title: layout_data.title,
            moveup_url: layout_data.moveup_url,
            username: layout_data.username,
        })
    }
}

#[derive(Default, Serialize)]
pub struct IndexLayoutData {
    title: String,
    username: String,
    moveup_url: String,
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
    pub fn moveup_url(mut self, moveup_url: &str) -> Self {
        self.moveup_url = moveup_url.to_owned();

        self
    }
}
