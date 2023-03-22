use chrono::{serde::ts_milliseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub why: String,
    pub note: String,
    #[serde(rename = "startAtMs")]
    #[schemars(with = "String")]
    #[serde(with = "ts_milliseconds")]
    pub start_at_ms: DateTime<Utc>,
    #[serde(rename = "endAtMs")]
    #[schemars(with = "String")]
    #[serde(with = "ts_milliseconds")]
    pub end_at_ms: DateTime<Utc>,
    pub finished: bool,
}

#[cfg(feature = "okapi")]
mod open_api_features {
    use super::*;
    use okapi::openapi3::Responses;
    use rocket_okapi::response::OpenApiResponderInner;

    impl OpenApiResponderInner for Challenge {
        fn responses(
            _: &mut rocket_okapi::gen::OpenApiGenerator,
        ) -> rocket_okapi::Result<okapi::openapi3::Responses> {
            Ok(Responses {
                ..Default::default()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime, Utc};

    use super::Challenge;

    #[test]
    fn can_stringify() {
        let _ = serde_json::to_string(&Challenge {
            id: "_id".to_owned(),
            title: "title".to_owned(),
            why: "why".to_owned(),
            note: "note".to_owned(),
            start_at_ms: DateTime::from_utc(NaiveDateTime::default(), Utc),
            end_at_ms: DateTime::from_utc(NaiveDateTime::default(), Utc),
            finished: false,
        })
        .unwrap();
    }

    #[test]
    fn can_parse_json() {
        let _: Challenge = serde_json::from_str(&"{\"id\":\"_id\",\"title\":\"title\",\"why\":\"why\",\"note\":\"note\",\"startedAt\":0,\"endAt\":0,\"finished\":false}").unwrap();
    }
}
