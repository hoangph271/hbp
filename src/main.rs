#![forbid(unsafe_code)]

// #region imports
#[macro_use]
extern crate dotenv_codegen;
extern crate mustache;
extern crate rocket_okapi;
extern crate serde_derive;

use log::{error, info, warn};
use rocket::{launch, routes};
use rocket_okapi::{
    mount_endpoints_and_merged_docs,
    settings::OpenApiSettings,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

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
    let mut rocket = rocket::build()
        .manage(sled::open("hbp.sled.db").unwrap())
        .mount("/", utils::cors::options_routes())
        .mount("/", routes::index::index_routes())
        .mount("/dev/null", routes::index::dev_null_routes())
        .mount("/markdown", routes::markdown::markdown_routes())
        .mount("/static", routes![routes::static_files::serve])
        .mount("/posts", routes::posts::posts_routes())
        .mount("/users", routes::users::users_routes())
        .mount("/blogs", routes![routes::blogs::index])
        .mount("/git", routes::git::git_routes())
        // * Swagger UI routes
        .mount(
            "/swagger",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        // * catchers
        .register("/", routes::catchers::catchers())
        .attach(utils::cors::Cors);

    let openapi_settings = OpenApiSettings::default();
    mount_endpoints_and_merged_docs! {
        rocket,
        "/api/v1",
        openapi_settings,
        "/markdowns" => routes::markdown::get_routes_and_docs(&openapi_settings),
        "/users" => routes::users::get_routes_and_docs(&openapi_settings),
        "/movies_and_tv" => routes::movies_and_tv::get_routes_and_docs(&openapi_settings),
        "/profiles" => routes::profiles::get_routes_and_docs(&openapi_settings),
        "/challenges" =>routes::challenges::get_routes_and_docs(&openapi_settings),
        "/files" => routes::files::get_routes_and_docs(&openapi_settings)
    };

    rocket
}
