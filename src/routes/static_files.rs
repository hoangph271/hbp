use rocket::{fs::NamedFile, get};
use std::path::{Path, PathBuf};

#[get("/<file..>")]
pub async fn serve(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).await.ok()
}
