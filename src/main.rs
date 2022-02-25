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

use dotenv::dotenv;
use routes::{index, markdown, static_files};
// #endregion

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");

    try_db();

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    utils::setup_logger::setup_logger();

    rocket::build()
        .mount("/", routes![index::index, index::readme_md])
        .mount("/markdown", routes![markdown::markdown_file])
        .mount("/static", routes![static_files::serve])
}

fn try_db() {
    use data::models::*;
    use data::schema::tbl_posts::dsl::*;
    use data::sqlite::establish_connection;
    use diesel::prelude::*;

    let connection = establish_connection();

    data::lib::create_post(&connection, "name", "age");

    let posts = tbl_posts
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Found {} post(s)...!", posts.len());
}
