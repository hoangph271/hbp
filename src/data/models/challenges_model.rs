#[derive(Debug, serde::Serialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub why: String,
    pub note: String,
    pub started_at: i64,
    pub end_at: i64,
    pub finished: bool,
}
