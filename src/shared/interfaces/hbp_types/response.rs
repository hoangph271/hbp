use super::utils::*;
use httpstatus::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct ApiList<T>
where
    T: Serialize,
{
    #[serde(rename = "statusCode")]
    #[serde(deserialize_with = "status_code_from_u16")]
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub items: Vec<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApiError {
    #[serde(rename = "statusCode")]
    #[serde(deserialize_with = "status_code_from_u16")]
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub errors: Vec<String>,
    #[serde(skip_serializing)]
    pub with_ui: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiItem<T: Serialize> {
    #[serde(rename = "statusCode")]
    #[serde(deserialize_with = "status_code_from_u16")]
    #[serde(serialize_with = "status_code_serialize")]
    pub status_code: StatusCode,
    pub item: T,
}

mod more_impls {
    use super::{ApiError, ApiItem};
    use httpstatus::StatusCode;
    use serde::Serialize;

    impl ApiError {
        pub fn new(status_code: StatusCode, errors: Vec<String>) -> Self {
            Self {
                status_code,
                errors,
                with_ui: false,
            }
        }

        pub fn bad_request(errors: Vec<String>) -> Self {
            ApiError {
                status_code: StatusCode::BadRequest,
                errors,
                with_ui: false,
            }
        }

        pub fn from_status(status_code: StatusCode) -> Self {
            Self {
                with_ui: false,
                status_code: status_code.clone(),
                errors: vec![status_code.reason_phrase().to_string()],
            }
        }

        pub fn from_message(msg: &str, status_code: StatusCode) -> ApiError {
            ApiError {
                with_ui: false,
                status_code,
                errors: vec![msg.to_owned()],
            }
        }

        pub fn unauthorized() -> ApiError {
            Self::from_status(StatusCode::Unauthorized)
        }

        pub fn not_implemented() -> ApiError {
            Self::from_status(StatusCode::NotImplemented)
        }

        pub fn not_found() -> ApiError {
            Self::from_status(StatusCode::NotFound)
        }

        pub fn forbidden() -> ApiError {
            Self::from_status(StatusCode::Forbidden)
        }

        pub fn unprocessable_entity() -> ApiError {
            Self::from_status(StatusCode::UnprocessableEntity)
        }

        pub fn internal_server_error() -> ApiError {
            Self::from_status(StatusCode::InternalServerError)
        }

        pub fn append_error(mut self, error: String) -> Self {
            self.errors.push(error);

            self
        }

        pub fn with_ui(mut self) -> Self {
            self.with_ui = true;
            self
        }
    }

    impl<T: Serialize> ApiItem<T> {
        pub fn ok(item: T) -> ApiItem<T> {
            ApiItem {
                status_code: StatusCode::Ok,
                item,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::ApiItem;

    #[test]
    fn can_stringify() {
        let _ = serde_json::to_string(&ApiItem::ok(vec!["0", "1", "2"])).unwrap();
    }

    #[test]
    fn can_parse_json() {
        let _: ApiItem<Vec<String>> =
            serde_json::from_str(r#"{"statusCode":200,"item":["0","1","2"]}"#).unwrap();
    }
}

#[cfg(feature = "okapi")]
mod open_api_features {
    use super::*;
    use okapi::openapi3::Responses;
    use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
    use serde::Serialize;

    impl<T: Serialize> OpenApiResponderInner for ApiList<T> {
        fn responses(
            _: &mut rocket_okapi::gen::OpenApiGenerator,
        ) -> rocket_okapi::Result<okapi::openapi3::Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }

    impl<T: Serialize> OpenApiResponderInner for ApiItem<T> {
        fn responses(_: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }

    impl OpenApiResponderInner for ApiError {
        fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }
}
