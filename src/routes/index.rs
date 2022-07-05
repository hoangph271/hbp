use rocket::http::Status;
use rocket::response::Redirect;

#[get("/README.md")]
pub fn readme_md() -> Redirect {
    Redirect::permanent("/markdown/README.md")
}

#[get("/")]
pub fn index() -> Redirect {
    Redirect::moved("/README.md")
}

#[get("/")]
pub fn get_dev_null() -> Status {
    Status::Ok
}
#[post("/")]
pub fn post_dev_null() -> Status {
    Status::Ok
}
#[put("/")]
pub fn put_dev_null() -> Status {
    Status::Ok
}
#[delete("/")]
pub fn delete_dev_null() -> Status {
    Status::Ok
}
