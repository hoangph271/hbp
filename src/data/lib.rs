use httpstatus::StatusCode;
use log::*;
use serde::Serialize;
use std::vec;
use thiserror::Error;

#[derive(Debug)]
#[allow(unused)]
pub enum OrmError {
    NotFound,
}

pub mod post_orm {
    use crate::data::models::posts_model::*;
    use crate::data::OrmInit;
    use rocket::async_trait;

    use super::{DbError, OrmError};

    pub fn get_one(_post_id: &str) -> Result<Post, OrmError> {
        todo!()
    }

    pub async fn get_posts() -> Result<Vec<Post>, DbError> {
        todo!()
    }

    pub fn delete_one(_post_id: &str) -> usize {
        todo!()
    }

    pub fn create_post(_new_post: NewPost) -> Result<Post, ()> {
        todo!()
    }

    pub fn update_one(_updated_post: UpdatedPost) -> Result<Post, OrmError> {
        todo!()
    }

    #[derive(Default)]
    pub struct PostOrm {}

    #[async_trait]
    impl OrmInit for PostOrm {
        fn db_file_name(&self) -> String {
            "posts.sled.db".to_owned()
        }
    }
}

use crate::{shared::interfaces::ApiError, utils::responders::HbpError};

#[derive(Error, Debug, Serialize)]
#[error("DbError: {status_code:?} - {message}")]
pub struct DbError {
    pub status_code: StatusCode,
    pub message: String,
}

pub type DbResult<T> = Result<T, DbError>;

impl DbError {
    pub fn internal_server_error(message: String) -> DbError {
        DbError {
            status_code: StatusCode::InternalServerError,
            message,
        }
    }
}
impl From<DbError> for HbpError {
    fn from(db_error: DbError) -> Self {
        ApiError::new(db_error.status_code, vec![db_error.message]).into()
    }
}
