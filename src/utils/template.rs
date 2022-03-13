use crate::utils::types::{HbpError, HbpResult};
use mustache::{Data, MapBuilder, Template};
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

pub fn render_from_template_by_default_page(
    template_path: &str,
    data: &Option<Data>,
) -> HbpResult<String> {
    let html = render_from_template(
        "index.html",
        &Some(
            MapBuilder::new()
                .insert_str(
                    "raw_content",
                    render_from_template(template_path, data).unwrap(),
                )
                .build(),
        ),
    );

    match html {
        Ok(html) => HbpResult::Ok(html),
        Err(e) => {
            debug!("{e}");
            HbpResult::Err(HbpError::from_message(&format!(
                "Failed render_from_template_by_default_page(), {template_path}"
            )))
        }
    }
}

pub fn data_from (fields: Vec<(String, String)>) -> mustache::Data {
    let mut builder = MapBuilder::new();

    for (key, value) in fields {
        builder = builder.insert_str(key, value);
    }

    builder.build()
}

pub type TemplateData = Vec<(String, String)>;
