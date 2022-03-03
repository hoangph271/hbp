use mustache::{Data, Error, MapBuilder, Template};
use std::path::Path;

pub fn compile_template(path: &str) -> Template {
    mustache::compile_path(Path::new("template").join(path))
        .unwrap_or_else(|_| panic!("compile template from {path} failed...!"))
}

pub fn render_from_template(template_path: &str, data: &Option<Data>) -> Result<String, Error> {
    let template = compile_template(template_path);

    if let Some(data) = data {
        template.render_data_to_string(data)
    } else {
        template.render_data_to_string(&MapBuilder::new().build())
    }
}
