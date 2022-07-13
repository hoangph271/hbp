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
    data::init_db().await;

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");

    launch()
}

fn launch() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![routes::index::index, routes::index::readme_md])
        .mount(
            "/api/movies_and_tv",
            routes![
                routes::movies_and_tv::get_all_shows,
                routes::movies_and_tv::get_one_show
            ],
        )
        .mount(
            "/dev/null",
            routes![
                routes::index::get_dev_null,
                routes::index::post_dev_null,
                routes::index::put_dev_null,
                routes::index::delete_dev_null
            ],
        )
        .mount(
            "/markdown",
            routes![
                routes::markdown::markdown_file,
                routes::markdown::user_markdown_file,
                routes::markdown::user_markdown_editor,
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
                routes::users::signup,
                routes::users::post_login,
                routes::users::post_signup,
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

    utils::responders::HbpResponse::status(StatusCode::from(status.code))
}
