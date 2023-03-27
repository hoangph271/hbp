use crate::{
    shared::interfaces::{ApiError, ApiItem},
    utils::{auth::AuthPayload, responders::HbpApiResult},
};
use async_std::fs::metadata;
use response_types::*;
use rocket::get;
use std::path::PathBuf;

#[get("/users/<username>/<sub_path..>")]
pub(super) async fn api_user_markdowns(
    username: &str,
    sub_path: PathBuf,
    jwt: AuthPayload,
) -> HbpApiResult<MarkdownItem> {
    jwt.assert_username(username)?;

    if sub_path.is_dir() {
        Err(ApiError::bad_request(vec![
            "markdown directory requests are NOT yet handled".to_owned()
        ])
        .into())
    } else {
        let metadata = metadata(&sub_path).await?;
        let markdown_item = MarkdownItem {
            filename: sub_path
                .file_name()
                .map(|filename| filename.to_string_lossy())
                .unwrap_or_else(|| sub_path.to_string_lossy())
                .to_string(),
            size: metadata.len(),
        };

        Ok(ApiItem::ok(markdown_item).into())
    }
}

mod response_types {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct MarkdownItem {
        pub filename: String,
        pub size: u64,
    }
}

#[cfg(test)]
mod markdown_api_tests {
    use super::*;
    use crate::utils::{auth::UserJwt, constants::cookies::USER_JWT};

    use httpstatus::StatusCode;
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

    #[test]
    fn rocket_startup_normally() {
        let client = get_client();

        let res = client.get("").dispatch();

        assert_eq!(res.status(), Status::BadRequest);
    }

    #[test]
    fn handle_unauthorized() {
        let client = get_client();

        let sub_path = PathBuf::from("");
        let res = client
            .get(uri!(api_user_markdowns(USERNAME, sub_path)))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    }

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
        let filename = "README.md";
        let sub_path = PathBuf::from(filename);
        let user_jwt = UserJwt {
            sub: USERNAME.to_owned(),
            ..Default::default()
        }
        .sign_jwt()
        .expect("sign_jwt() failed");

        let client = get_client();
        let res = client
            .get(uri!(api_user_markdowns(USERNAME, sub_path)))
            .cookie(Cookie::new(USER_JWT, user_jwt))
            .dispatch()
            .into_json::<ApiItem<MarkdownItem>>()
            .unwrap_or_else(|| panic!("res.into_json() failed"));

        assert_eq!(res.status_code, StatusCode::Ok);
        assert_eq!(res.item.filename, filename);
    }
}
