use rocket::{delete, get, http::Status, post, put, routes, Route};

#[get("/")]
fn get_dev_null() -> Status {
    Status::Ok
}
#[post("/")]
fn post_dev_null() -> Status {
    Status::Ok
}
#[put("/")]
fn put_dev_null() -> Status {
    Status::Ok
}
#[delete("/")]
fn delete_dev_null() -> Status {
    Status::Ok
}

pub fn dev_null_routes() -> Vec<Route> {
    routes![get_dev_null, post_dev_null, put_dev_null, delete_dev_null]
}
