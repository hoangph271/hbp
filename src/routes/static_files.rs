use crate::utils::{
    constants,
    jwt::JwtPayload,
    responders::{HbpContent, HbpResponse},
    template,
};
use httpstatus::StatusCode;
use mustache::{Data, MapBuilder};
use rocket::http::ContentType;
use rocket::tokio::fs::File;
use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct DirEntry {
    name: String,
    path: String,
}

fn dir_entries_map(path: &Path) -> Data {
    MapBuilder::new()
        .insert_vec("dir_entries", |builder| {
            let entries = fs::read_dir(path).ok().unwrap();

            entries.fold(builder, |builder, entry| {
                let entry = entry.unwrap();
                let dir_entry = DirEntry {
                    name: entry.file_name().into_string().unwrap(),
                    path: entry.path().to_str().unwrap().to_owned(),
                };

                builder.push(&dir_entry).unwrap()
            })
        })
        .build()
}

#[get("/")]
pub async fn static_dir(_jwt: JwtPayload) -> Option<HbpResponse> {
    let data = dir_entries_map(constants::static_path());
    let html = template::render_from_template("static/directory.html", &data).ok()?;

    let content = HbpContent::Html(html);
    Some(HbpResponse::ok(content))
}

#[get("/bin/<path..>")]
pub async fn bin(path: PathBuf) -> Option<HbpResponse> {
    let full_path = constants::static_path().join(path.clone());
    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    if fs::metadata(&full_path).ok()?.is_dir() {
        let tempfile = tempfile::tempfile().ok()?;
        let mut zip_writer = zip::ZipWriter::new(tempfile);
        let mut buffer = Vec::new();

        walkdir::WalkDir::new(&full_path)
            .into_iter()
            // TODO: Failed files...?
            .filter_map(|entry| entry.ok())
            .for_each(|entry| {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy();

                if path.is_file() {
                    zip_writer.start_file(name, options).unwrap();

                    let mut file = fs::File::open(path).unwrap();

                    file.read_to_end(&mut buffer).unwrap();
                    zip_writer.write_all(&buffer).unwrap();
                    buffer.clear();
                } else {
                    // ? I do NOT thing this is working
                    // ? Skip for now anyway
                    // #[allow(deprecated)]
                    // zip_writer.add_directory_from_path(path, options).unwrap();
                }
            });

        zip_writer.finish().ok()?;

        // return Some(HbpResponse::ok(HbpContent::Sized(
        //     Box::new(ContentType::ZIP),
        //     Box::new(tempfile.read()),
        // )));
    }

    let ext = full_path
        .extension()
        .unwrap_or_else(|| OsStr::new(""))
        .to_string_lossy();
    let content_type = ContentType::from_extension(&ext).unwrap_or(ContentType::Any);

    let file = File::open(full_path).await.ok()?;

    Some(HbpResponse::ok(HbpContent::File(
        Box::new(content_type),
        file,
    )))
}

#[get("/<path..>", rank = 1)]
pub async fn dir(path: PathBuf, _jwt: JwtPayload) -> Option<HbpResponse> {
    let full_path = constants::static_path().join(path.clone());

    if fs::metadata(&full_path).ok()?.is_dir() {
        let data = dir_entries_map(&full_path);
        let html = template::render_from_template("static/directory.html", &data).ok()?;

        return Some(HbpResponse::ok(HbpContent::Html(html)));
    }

    Some(HbpResponse::status_text(
        StatusCode::BadRequest,
        &format!("{} is NOT a directory", path.to_string_lossy()),
    ))
}
