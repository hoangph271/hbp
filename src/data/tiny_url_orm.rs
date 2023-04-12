use rocket::async_trait;
use serde::__private::from_utf8_lossy;

use super::models::tiny_url::TinyUrl;
use super::{lib::DbError, OrmInit};

#[derive(Default)]
pub struct TinyUrlOrm {}

#[async_trait]
impl OrmInit for TinyUrlOrm {
    fn db_file_name(&self) -> String {
        "tiny_urls.sled.db".to_owned()
    }
}

impl TinyUrlOrm {
    pub async fn find_one(
        &self,
        db: &sled::Db,
        slug: &str,
    ) -> Result<Option<TinyUrl>, DbError> {
        if let Some(raw) = db.get(slug).unwrap() {
            let json = from_utf8_lossy(&raw[..]);
            Ok(serde_json::from_str(&json).ok())
        } else {
            Ok(None)
        }
    }

    pub async fn create_tiny_url(
        &self,
        db: &sled::Db,
        tiny_url: TinyUrl,
    ) -> Result<TinyUrl, DbError> {
        let slug = tiny_url.slug.clone();

        db.insert(
            tiny_url.slug.clone(),
            serde_json::to_string(&tiny_url).unwrap().as_bytes(),
        )
        .unwrap();

        self.find_one(db, &slug)
            .await
            .unwrap()
            .ok_or(DbError::internal_server_error(
                "create Challenge failed".to_string(),
            ))
    }
}
