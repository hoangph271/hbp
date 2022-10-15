use std::path::PathBuf;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Status},
    options, routes, Request, Response, Route,
};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS, PUT",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<_path..>")]
fn options(_path: PathBuf) -> Status {
    Status::Ok
}

pub fn options_routes() -> Vec<Route> {
    routes![options]
}
