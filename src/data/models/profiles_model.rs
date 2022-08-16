use serde::Serialize;
use stargate_grpc_derive::TryFromRow;

#[derive(Serialize, TryFromRow)]
pub struct DbProfile {
    username: String,
    title: String,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
}
