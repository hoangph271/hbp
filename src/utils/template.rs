use crate::utils::types::{HbpError, HbpResult};
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

pub fn render_from_template(template_path: &str, data: &Option<Data>) -> HbpResult<String> {
    let template = compile_template(PathBuf::from(template_path));

    let render_result = if let Some(data) = data {
        template.render_data_to_string(data)
    } else {
        template.render_data_to_string(&MapBuilder::new().build())
    };

    match render_result {
        Ok(data) => HbpResult::Ok(data),
        Err(e) => {
            error!("{e}");
            HbpResult::Err(HbpError::from_message(&format!(
                "Failed render_from_template(), {template_path}, {:?}",
                data
            )))
        }
    }
}

#[derive(Default)]
pub struct DefaultLayoutData {
    title: Option<String>,
    username: Option<String>,
}
impl DefaultLayoutData {
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());

        self
    }
    pub fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_owned());

        self
    }
    pub fn only_title(title: &str) -> Self {
        Self::default().title(title)
    }
}
pub fn render_default_layout(
    template_path: &str,
    layout_data: Option<DefaultLayoutData>,
    data: &Option<Data>,
) -> HbpResult<String> {
    let layout_data = layout_data.unwrap_or_default();

    let html = render_from_template(
        "index.html",
        &Some(
            MapBuilder::new()
                .insert_str(
                    "raw_content",
                    render_from_template(template_path, data).unwrap(),
                )
                .insert_str("title", layout_data.title.unwrap_or_else(|| "".to_owned()))
                .insert_str(
                    "username",
                    layout_data.username.unwrap_or_else(|| "".to_owned()),
                )
                .build(),
        ),
    );

    match html {
        Ok(html) => HbpResult::Ok(html),
        Err(e) => {
            debug!("{e}");
            HbpResult::Err(HbpError::from_message(&format!(
                "Failed render_default_layout(), {template_path}"
            )))
        }
    }
}

pub fn data_from(fields: TemplateData) -> Data {
    let mut builder = MapBuilder::new();

    fn insert_map(mut builder: MapBuilder, values: &HashMap<String, Data>) -> MapBuilder {
        for (key, value) in values {
            builder = match value {
                Data::String(value) => builder.insert_str(key, value),
                Data::Map(values) => insert_map(builder, values),
                _ => builder,
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
