use mustache::{Data, Error, MapBuilder, Template};
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

pub fn render_from_template(template_path: &str, data: &Option<Data>) -> Result<String, Error> {
    let template = compile_template(PathBuf::from(template_path));

    if let Some(data) = data {
        template.render_data_to_string(data)
    } else {
        template.render_data_to_string(&MapBuilder::new().build())
    }
}

pub fn render_from_template_paged(
    template_path: &str,
    data: &Option<Data>,
) -> Result<String, Error> {
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
    )
    .unwrap();

    Ok(html)
}
