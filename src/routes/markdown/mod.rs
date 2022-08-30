mod api;
mod ui;

use std::path::{Path, PathBuf};

pub use api::*;
pub use ui::*;

use crate::utils::auth::UserJwt;

fn assert_payload_access(payload: &UserJwt, path: &Path) -> bool {
    let prefix = PathBuf::from("markdown")
        .join("users")
        .join(payload.sub.clone())
        .to_string_lossy()
        .into_owned();

    path.starts_with(&*prefix)
}

fn markdown_path_from(username: &str, sub_path: &Path) -> (String, PathBuf) {
    let file_path = PathBuf::from("markdown")
        .join("users")
        .join(username)
        .join(sub_path);

    (file_path.to_string_lossy().to_string(), file_path)
}
