use crate::utils::jwt::JwtPayload;
use crate::utils::responders::{HbpContent, HbpResponse};

#[get("/")]
pub fn index(jwt: JwtPayload) -> HbpResponse {
    return HbpResponse::text(&*format!("hello {:?}", jwt), httpstatus::StatusCode::Ok);
}

#[get("/login")]
pub fn login() -> HbpResponse {
    HbpResponse::ok(Some(HbpContent::Plain("login".to_owned())))
}
