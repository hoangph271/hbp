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

    // let conn = data::sqlite::establish_connection();
    // data::lib::user_orm::create_user(&conn, data::models::users_model::NewUser {
    //     username: "username",
    //     hashed_password: "hashed_password",
    //     title: Some("tilte")
    // });

    rocket::build()
        .mount("/", routes![routes::index::index, routes::index::readme_md])
        .mount("/markdown", routes![routes::markdown::markdown_file])
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
}
