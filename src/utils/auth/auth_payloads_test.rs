use crate::utils::timestamp_now;

use super::{AuthPayload, UserJwt};

#[test]
fn parse_jwt_from_str() {
    let jwt_str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJoYnAiLCJleHAiOjI1MTYyMzkwMjIsInBhdGgiOiIifQ.VTzEo0XgzK5kV5WySIY0JYvB7h-uHDQ4alAnEb48JbQ";

    if let AuthPayload::UserResource(claims) =
        AuthPayload::decode(jwt_str).unwrap_or_else(|e| panic!("jwt_str MUST be a JWT: {e:?}"))
    {
        assert_eq!(claims.sub, "hbp");
        assert!(claims.exp > timestamp_now());
    } else {
        panic!("Must be parsed into AuthPayload::UserResource");
    }
}

#[test]
fn create_jwt_str_and_parse_again() {
    let jwt_str = UserJwt::default()
        .set_sub("hbp".to_owned())
        .sign_jwt()
        .unwrap_or_else(|e| panic!("sign_jwt() must works, got: {e:?}"));

    assert!(AuthPayload::decode(&jwt_str).is_ok());
}
