use crate::utils::auth::AuthPayload;
use crate::utils::types::{HbpError, HbpResult};
use httpstatus::StatusCode;
use log::*;
use mustache::{Data, MapBuilder, Template};
use std::collections::hash_map::HashMap;
use std::path::{Path, PathBuf};

pub fn compile_template(path: PathBuf) -> Template {
    mustache::compile_path(Path::new("template").join(path.clone())).unwrap_or_else(|e| {
        error!("{e}");

        panic!(
            "compile template from {} failed...!",
            path.to_string_lossy()
        )
    })
}

pub fn render_from_template(template_path: &str, data: Option<Data>) -> HbpResult<String> {
    let template = compile_template(PathBuf::from(template_path));

    let render_result = if let Some(data) = data {
        template.render_data_to_string(&data)
    } else {
        template.render_data_to_string(&MapBuilder::new().build())
    };

    match render_result {
        Ok(data) => HbpResult::Ok(data),
        Err(e) => {
            error!("{e}");
            HbpResult::Err(HbpError::from_message(
                &format!("Failed render_from_template(), {template_path}"),
                StatusCode::InternalServerError,
            ))
        }
    }
}

#[derive(Default)]
pub struct DefaultLayoutData {
    title: String,
    username: String,
    moveup_url: String,
}
impl DefaultLayoutData {
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
pub fn render_default_layout(
    template_path: &str,
    layout_data: Option<DefaultLayoutData>,
    data: Option<Data>,
) -> HbpResult<String> {
    let mut template_data = MapBuilder::new();

    if let Some(layout_data) = layout_data {
        template_data = template_data.insert_str("title", layout_data.title);
        template_data = template_data.insert_str("moveup_url", layout_data.moveup_url);
        template_data = template_data.insert_str(
            "raw_content",
            render_from_template(template_path, data).unwrap(),
        );

        if !layout_data.username.is_empty() {
            template_data = template_data.insert_str("username", layout_data.username);
        }
    }

    let html = render_from_template("index.html", Some(template_data.build()));

    match html {
        Ok(html) => HbpResult::Ok(html),
        Err(e) => {
            debug!("{e}");
            HbpResult::Err(HbpError::from_message(
                &format!("Failed render_default_layout(), {template_path}"),
                StatusCode::InternalServerError,
            ))
        }
    }
}

pub fn simple_data_from(fields: TemplateData) -> Data {
    let mut builder = MapBuilder::new();

    fn insert_map(mut builder: MapBuilder, values: &HashMap<String, Data>) -> MapBuilder {
        for (key, value) in values {
            builder = match value {
                Data::String(value) => builder.insert_str(key, value),
                Data::Map(values) => insert_map(builder, values),
                _ => panic!(),
            }
        }

        builder
    }

    for (key, value) in fields {
        builder = match value {
            Data::String(value) => builder.insert_str(key, value),
            Data::Map(values) => builder.insert_map(key, |builder| insert_map(builder, &values)),
            _ => builder,
        }
    }

    builder.build()
}

pub type TemplateData = Vec<(String, Data)>;
