use httpstatus::StatusCode;
use reqwest::Client;
use rocket::{get, routes, Route};
use serde::{Deserialize, Serialize};

use crate::utils::{responders::{HbpResponse, HbpResult}, template::{Templater, IndexLayout}};

#[derive(Deserialize, Serialize, Debug)]
struct NftCollection {
    name: String,
    image_url: String,
    slug: String,
}

const API_ROOT: &str = "https://api.opensea.io/api/v1";
const ASSET_OWNER: &str = "0x09c7b1F6a75b56065061aE15bd93e3F492c4efB9";

#[get("/")]
async fn all_galleries() -> HbpResult<HbpResponse> {
    #[derive(Serialize, Deserialize, Debug)]
    struct CollectionsResponse {
        collections: Vec<NftCollection>,
    }

    let url = format!("{}/collections?asset_owner={}", API_ROOT, ASSET_OWNER);
    let html = Templater::new("gallery/list.html".into()).to_html_page(
        CollectionsResponse {
            collections: Client::new().get(url).send().await?.json().await?,
        },
        IndexLayout::default(),
    )?;

    Ok(HbpResponse::html(html, StatusCode::Ok))
}

pub fn nfs_gallery_routes() -> Vec<Route> {
    routes![all_galleries]
}
