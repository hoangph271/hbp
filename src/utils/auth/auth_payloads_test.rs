use super::{jwt, AuthPayload, UserPayload};
use super::super::timestamp_now;

#[test]
fn parse_jwt_from_str() {
    let jwt_str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJoYnAiLCJleHAiOjI1MTYyMzkwMjIsInBhdGgiOiIifQ.VTzEo0XgzK5kV5WySIY0JYvB7h-uHDQ4alAnEb48JbQ";

    if let AuthPayload::UserResource(claims) = jwt::verify_jwt(jwt_str).unwrap() {
        assert_eq!(claims.sub, "hbp");
        assert!(claims.exp > 0);
    } else {
        panic!("Must be parsed into AuthPayload::UserResource");
    }
}

#[test]
fn create_jwt_str_and_parse_again() {
    use crate::utils::setup_logger;
    setup_logger::setup_logger();

    let jwt_str = UserPayload::sign_jwt(&UserPayload {
        exp: timestamp_now(),
        sub: "hbp".to_owned(),
        role: vec![],
    });

    jwt::verify_jwt(&jwt_str).unwrap();
}
