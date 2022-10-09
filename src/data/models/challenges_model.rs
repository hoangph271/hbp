use stargate_grpc_derive::TryFromRow;

#[derive(Debug, serde::Serialize, TryFromRow)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub why: String,
    pub note: String,
    pub started_at: u64,
    pub end_at: u64,
    pub finished: bool,
}
