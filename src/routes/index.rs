#[get("/README.md")]
pub fn readme_md() -> rocket::response::Redirect {
    rocket::response::Redirect::permanent("/markdown/README.md")
}

#[get("/")]
pub fn index() -> rocket::response::Redirect {
    rocket::response::Redirect::moved("/README.md")
}
