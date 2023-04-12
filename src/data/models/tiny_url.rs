use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TinyUrl {
    pub id: String,
    pub slug: String,
    #[serde(rename = "fullUrl")]
    pub full_url: String
}
