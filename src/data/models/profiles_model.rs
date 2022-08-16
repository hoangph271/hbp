use serde::Serialize;
use stargate_grpc_derive::{IntoValues, TryFromRow};

#[derive(Serialize, TryFromRow, Clone, IntoValues)]
pub struct DbProfile {
    pub username: String,
    pub title: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}
