// #region imports
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate dotenv_codegen;
extern crate mustache;
extern crate serde_derive;

mod routes;
mod utils;

use routes::{index, markdown, static_files};
// #endregion

#[launch]
fn rocket() -> _ {
    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    utils::setup_logger::setup_logger();

    rocket::build()
        .mount("/", routes![index::index, index::readme_md])
        .mount("/markdown", routes![markdown::markdown_file])
        .mount(
            "/static",
            routes![
                static_files::static_dir,
                static_files::dir,
                static_files::bin
            ],
        )
}
