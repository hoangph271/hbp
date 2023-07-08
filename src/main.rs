#![forbid(unsafe_code)]

// #region imports
#[macro_use]
extern crate dotenv_codegen;
extern crate mustache;
extern crate serde_derive;

use log::{error, info, warn};
use rocket::{fs::FileServer, launch, routes};

use crate::utils::env::{from_env, EnvKey};

mod data;
mod routes;
mod shared;
mod utils;
// #endregion

#[launch]
async fn rocket() -> _ {
    utils::setup_logger::setup_logger();

    dotenv::dotenv().unwrap_or_else(|e| {
        error!("dotenv() failed: {}", e);
        panic!()
    });

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    info!("{app_name} is starting, my dude...! 🍿🍿🍿");

    if utils::env::is_prod() {
        warn!("{app_name} is running IN PRODUCTION");
    }

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    let rocket = rocket::build()
        .manage(sled::open("hbp.sled.db").expect("hbp.sled.db doesn't exist...!"))
        .mount("/", utils::cors::options_routes())
        .mount("/", routes::index::index_routes())
        .mount("/ui", FileServer::from(from_env(EnvKey::SneuUiRoot)))
        .mount("/dev/null", routes::index::dev_null_routes())
        .mount("/markdown", routes::markdown::markdown_routes())
        .mount("/static", routes![routes::static_files::serve])
        .mount("/users", routes::users::users_routes())
        .mount("/blogs", routes![routes::blogs::index])
        .mount("/gallery", routes::nft_gallery::nfs_gallery_routes())
        .mount("/git", routes::git::git_routes())
        .mount("/tiny", routes::tiny_urls::tiny_urls_routes())
        // * API routes
        .mount("/api/v1/markdowns", routes::markdown::markdown_api_routes())
        .mount("/api/v1/users", routes::users::users_api_routes())
        .mount(
            "/api/v1/movies_and_tv",
            routes::movies_and_tv::movies_and_tv_api_routes(),
        )
        .mount("/api/v1/profiles", routes::profiles::profiles_api_routes())
        .mount("/api/v1/files", routes::files::files_api_routes())
        // * catchers
        .register("/", routes::catchers::catchers())
        .attach(utils::cors::Cors);

    rocket
}
