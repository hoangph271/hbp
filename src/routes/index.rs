use crate::utils::{
    responders::{HbpContent, HbpResponse},
    template,
};
use httpstatus::StatusCode;
use mustache::MapBuilder;

#[get("/README.md")]
pub fn readme_md() -> rocket::response::Redirect {
    rocket::response::Redirect::permanent("/markdown/README.md")
}

#[get("/")]
pub fn index() -> HbpResponse {
    let data = MapBuilder::new()
        .insert_str("title", "@HBP")
        .insert_str(
            "raw_content",
            r#"
            <h3>
                Hello, it's me @HHP...!
            </h3>
            <p>
                Click <a href="/markdown/README.md">here</a> to read the README.md file...!
            </p>"#,
        )
        .build();

    match template::render_from_template("index.html", &Some(data)) {
        Ok(html) => HbpResponse::ok(Some(HbpContent::Html(html))),
        Err(e) => {
            error!("{e}");
            HbpResponse::status(StatusCode::InternalServerError)
        }
    }
}
