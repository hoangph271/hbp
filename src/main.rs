// #region imports
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate dotenv_codegen;
extern crate mustache;
extern crate serde_derive;

mod data;
mod routes;
mod utils;
// #endregion

#[launch]
fn rocket() -> _ {
    dotenv::dotenv().ok();

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    utils::setup_logger::setup_logger();

    rocket::build()
        .attach(data::sqlite::DbConn::fairing())
        .mount("/", routes![routes::index::index, routes::index::readme_md])
        .mount(
            "/markdown",
            routes![
                routes::markdown::markdown_file,
                routes::markdown::user_markdown_file
            ],
        )
        .mount("/static", routes![routes::static_files::serve])
        .mount(
            "/posts",
            routes![
                routes::posts::index,
                routes::posts::find_one,
                routes::posts::delete_one,
                routes::posts::create,
                routes::posts::update
            ],
        )
        .mount(
            "/users",
            routes![
                routes::users::index,
                routes::users::login,
                routes::users::post_login
            ],
        )
        .mount("/blogs", routes![routes::blogs::index])
        .register("/", catchers![default_catcher])
}

#[catch(default)]
fn default_catcher(
    status: rocket::http::Status,
    _req: &rocket::Request,
) -> utils::responders::HbpResponse {
    use httpstatus::StatusCode;
    use mustache::MapBuilder;
    use utils::responders::{HbpContent, HbpResponse};
    use utils::template::render_from_template_by_default_page;

    let status_code = StatusCode::from(status.code);

    let error_text = format!("{} | {}", status_code.as_u16(), status_code.reason_phrase());
    let html = render_from_template_by_default_page(
        "static/error.html",
        &Some(
            MapBuilder::new()
                .insert_str("error_text", error_text)
                .build(),
        ),
    )
    .unwrap();

    HbpResponse {
        status_code,
        content: HbpContent::Html(html),
    }
}
