use crate::{
    shared::interfaces::{ApiItem, ApiResult},
    utils::auth::AuthPayload,
};

use rocket::get;
use rocket_okapi::openapi;
use std::path::PathBuf;

#[openapi]
#[get("/users/<username>/<sub_path..>")]
pub(super) async fn api_user_markdowns(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
) -> ApiResult<ApiItem<String>> {
    jwt.assert_username(username)?;

    todo!()
}

#[cfg(test)]
mod markdown_api_tests {
    use super::*;
    use crate::utils::{auth::UserJwt, constants::cookies::USER_JWT};

    use rocket::{
        self,
        http::{Cookie, Status},
        local::blocking::Client,
        routes, uri,
    };
    use std::path::PathBuf;

    const USERNAME: &str = "username";

    fn get_client() -> Client {
        let rocket = rocket::build().mount("/", routes![api_user_markdowns]);
        Client::tracked(rocket).unwrap_or_else(|e| panic!("Client::tracked() failed: {e:?}"))
    }

    #[ignore]
    #[test]
    fn rocket_startup_normally() {
        let client = get_client();

        let res = client.get("").dispatch();

        assert_eq!(res.status(), Status::BadRequest);
    }

    #[ignore]
    #[test]
    fn handle_unauthorized() {
        let client = get_client();

        let sub_path = PathBuf::from("");
        let res = client
            .get(uri!(api_user_markdowns(USERNAME, sub_path)))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[ignore]
    #[test]
    fn handle_username_mismatch() {
        let client = get_client();

        let sub_path = PathBuf::from("");
        let user_jwt = UserJwt::default().sign_jwt().expect("sign_jwt() failed");

        let res = client
            .get(uri!(api_user_markdowns(USERNAME, sub_path)))
            .cookie(Cookie::new(USER_JWT, user_jwt))
            .dispatch();

        assert_eq!(res.status(), Status::Forbidden);
    }

    #[test]
    fn api_user_markdowns_works() {
        let client = get_client();

        let sub_path = PathBuf::from("");
        let user_jwt = UserJwt {
            sub: USERNAME.to_owned(),
            ..Default::default()
        }
        .sign_jwt()
        .expect("sign_jwt() failed");

        let res = client
            .get(uri!(api_user_markdowns(USERNAME, sub_path)))
            .cookie(Cookie::new(USER_JWT, user_jwt))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    }
}
