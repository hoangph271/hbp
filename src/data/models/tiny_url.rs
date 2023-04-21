use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct TinyUrl {
    slug: String,
    #[serde(rename = "fullUrl")]
    full_url: String,
}

impl TinyUrl {
    pub fn slug_from(full_url: &str) -> String {
        let mut sha256 = Sha256::new();
        sha256.update(full_url);

        format!("{:X}", sha256.finalize())
    }

    pub fn new(full_url: String) -> Self {
        Self {
            slug: Self::slug_from(&full_url),
            full_url,
        }
    }

    pub fn get_slug(&self) -> String {
        self.slug.to_string()
    }

    pub fn get_full_url(&self) -> String {
        self.full_url.to_string()
    }
}
