pub fn url_encode_path(path: &str) -> String {
    path.split('/')
        .map(|part| urlencoding::encode(part).to_string())
        .collect::<Vec<String>>()
        .join("/")
}
