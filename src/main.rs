// #region imports
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate dotenv_codegen;
extern crate mustache;
extern crate serde_derive;

mod data;
mod routes;
mod shared;
mod utils;
// #endregion

#[launch]
async fn rocket() -> _ {
    utils::setup_logger::setup_logger();

    dotenv::dotenv().ok();
    data::init_db();

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes::index::base_routes())
        .mount("/dev/null", routes::index::dev_null_routes())
        .mount("/markdown", routes::markdown::markdown_routes())
        .mount("/static", routes![routes::static_files::serve])
        .mount("/posts", routes::posts::posts_routes())
        .mount("/users", routes::users::users_routes())
        .mount("/blogs", routes![routes::blogs::index])
        // * API routes
        .mount(
            "/api/movies_and_tv",
            routes::movies_and_tv::api_movies_and_tv_routes(),
        )
        .mount("/api/users", routes::users::api_users_routes())
        // * catchers
        .register("/", routes::catchers::catchers())
}
