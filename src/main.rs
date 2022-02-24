#[macro_use]
extern crate rocket;
extern crate mustache;
extern crate serde_derive;

mod routes;
mod utils;

use routes::{index, markdown, static_files};

#[launch]
fn rocket() -> _ {
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
